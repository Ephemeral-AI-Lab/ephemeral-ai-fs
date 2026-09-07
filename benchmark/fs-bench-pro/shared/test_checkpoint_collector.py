"""Checkpoint resume must preserve identities and never pre-create proof output."""
import json
from pathlib import Path
import sys
import tempfile
import subprocess
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import issue54_collect as collect


class CheckpointCollectorTest(unittest.TestCase):
    def test_only_prework_refusal_is_retried_and_archived(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'proof'
            output.mkdir()
            data = json.dumps({'status': 'INCOMPLETE', 'error': 'RuntimeError: ' + collect.LOCK_REFUSAL,
                               'source_identity': None, 'checks': []})
            (output / 'verification.json').write_text(data)
            self.assertTrue(collect.retain_lock_refusal(output))
            self.assertFalse(output.exists())
            archived = next(Path(directory).glob('proof.lock-refused-*/verification.json'))
            self.assertEqual(archived.read_text(), data)
            command = ['runner', '--output', str(output)]
            with patch.object(collect.fcntl, 'flock'), patch.object(collect.subprocess, 'run', side_effect=[
                subprocess.CompletedProcess(command, 2, '', collect.LOCK_REFUSAL),
                subprocess.CompletedProcess(command, 0, 'done', ''),
            ]) as run:
                self.assertEqual(collect._run(command).returncode, 0)
                self.assertEqual(run.call_count, 2)
            with patch.object(collect.fcntl, 'flock'), patch.object(collect.subprocess, 'run', return_value=
                    subprocess.CompletedProcess(command, 1, '', 'workload timeout')) as run:
                self.assertEqual(collect._run(command).returncode, 1)
                self.assertEqual(run.call_count, 1)

    def test_sdk_resume_binding_and_immutable_proof_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            family, case = 'edit_length_preserving', 'test-sdk'
            row = {'scenario_id': case, 'route': 'sdk'}
            identity = {'family': family, 'case': case, 'source_identity': 'source',
                        'image': 'image', 'product_identity': 'product', 'input_identity': 'input',
                        'harness_identity': collect.runner.harness_identity(), 'setup_identity': 'clone'}
            sample = {'kind': 'sample', 'status': 'PASS', 'identities': identity,
                      'records': [{'row_id': 'sdk-row-1', 'edit_commit_ns': 100}]}
            path = root / 'perf.jsonl'
            path.write_text(json.dumps(sample) + '\n' + json.dumps({'kind': 'summary', 'status': 'PASS'}) + '\n')
            loaded = collect.load_performance(path, family, row, 'source', 'image')
            self.assertEqual(loaded['elapsed_ns'], 100)
            self.assertEqual(loaded['identities']['performance_rows'], 'sdk-row-1')
            with self.assertRaises(ValueError):
                collect.load_performance(path, family, row, 'different-source', 'image')
            args = SimpleNamespace(image='image', host_binary='host')
            def execute(argv):
                output = Path(argv[argv.index('--output') + 1])
                self.assertFalse(output.exists())
                self.assertIn('--repetition', argv)
                self.assertEqual(argv[argv.index('--performance-rows') + 1], 'sdk-row-1')
                output.mkdir()
                (output / 'verification.json').write_text(json.dumps({**identity, 'image_identity': 'image', 'status': 'PASS'}))
                return SimpleNamespace(returncode=0, stdout='', stderr='')
            with patch.object(collect, '_run', side_effect=execute):
                result = collect.verify_row(args, family, row, root, loaded['identities'])
            self.assertEqual(result['status'], 'PASS')
            with self.assertRaises(ValueError):
                collect.verify_row(args, family, row, root, {**loaded['identities'], 'input_identity': 'wrong'})


if __name__ == '__main__':
    unittest.main()
