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
    def test_campaign_rejects_contract_drift_before_execution(self):
        import copy
        root = collect.REPO / "docs/roadmap/0.1/0.1.4/issue91-campaign"
        declaration = json.loads((root / "declaration.json").read_text())
        rows = [json.loads(line) for line in (root / "registry.jsonl").read_text().splitlines()]
        families = {family: ([row for row in rows if row["family_id"] == family], {})
                    for family in declaration["families"]}
        collect.validate_campaign(declaration, families)
        for mutation in ("missing", "duplicate", "order", "fixture", "count", "timer", "exclusion", "family"):
            with self.subTest(mutation=mutation):
                spec, selected = copy.deepcopy(declaration), copy.deepcopy(families)
                members = next(iter(selected.values()))[0]
                if mutation == "missing": members.pop()
                elif mutation == "duplicate": members[1] = members[0]
                elif mutation == "order": members.reverse()
                elif mutation == "fixture": members[0]["fixture_bytes"] += 1
                elif mutation == "count": next(iter(spec["families"].values()))["performance"] += 1
                elif mutation == "timer": spec["verification_hard_seconds"] += 1
                elif mutation == "exclusion": spec["long_test_reason"] = ""
                elif mutation == "family": members[0]["family_id"] = "wrong"
                with self.assertRaises(ValueError):
                    collect.validate_campaign(spec, selected)

    def test_campaign_exclusion_is_explicit_and_does_not_execute(self):
        root = collect.REPO / "docs/roadmap/0.1/0.1.4/issue91-campaign"
        spec = json.loads((root / "declaration.json").read_text())
        args = SimpleNamespace(campaign_spec=spec)
        row = {"scenario_id": spec["long_test_exclusion"], "verification_supported": False}
        with tempfile.TemporaryDirectory() as directory, patch.object(collect, "_run") as run:
            result = collect.verify_row(args, "workspace_reliability", row, Path(directory), None)
            self.assertEqual(result["status"], "NOT_RUN_OPTIONAL")
            self.assertEqual(result["omissions"], [spec["long_test_reason"]])
            run.assert_not_called()
            with self.assertRaises(ValueError):
                collect.verify_row(args, "workspace_reliability", {**row, "scenario_id": "undeclared"},
                                   Path(directory), None)

    def test_independent_preparation_preserves_receipt_and_rejects_drift(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            declaration = root / "declaration.json"
            spec = {"schema": "issue91-proof-preparation-v1", "cases": ["one"],
                    "setup_timeout_seconds": 600, "verification_work_seconds": 45,
                    "verification_hard_seconds": 59}
            declaration.write_text(json.dumps(spec))
            row = {"scenario_id": "one", "setup_policy": "fresh-output", "route": "namespace"}
            collect.validate_proof_preparation(spec, {"init_namespace": ([row], {})})
            for changed in ({**spec, "cases": ["one", "one"]}, {**spec, "cases": ["absent"]},
                            {**spec, "verification_work_seconds": 46}):
                with self.assertRaises(ValueError):
                    collect.validate_proof_preparation(changed, {"init_namespace": ([row], {})})
            with self.assertRaises(ValueError):
                collect.validate_proof_preparation(spec, {"init_namespace": ([{**row, "setup_policy": "clone"}], {})})
            args = SimpleNamespace(proof_preparation_declaration=declaration, image="image", host_binary="host")
            identity = {"source_identity": "source", "product_identity": "product", "input_identity": "input", "image": "image"}
            actual = {**identity, "family": "init_namespace", "case": "one", "seed": 1,
                      "setup_identity": "fresh-output", "harness_identity": collect.runner.harness_identity()}
            payload = json.dumps({"status": "PASS", "identities": actual, "cleanup": {"status": "PASS"}})
            with patch.object(collect, "_run", return_value=SimpleNamespace(returncode=0, stdout=payload, stderr="")) as run:
                result = collect.prepare_proof(args, "init_namespace", row, root, identity)
                self.assertEqual(result["status"], "PASS")
                self.assertIn("--prepare-only", run.call_args.args[0])
                self.assertEqual(collect.prepare_proof(args, "init_namespace", row, root, identity), result)
                self.assertEqual(run.call_count, 1)
                with self.assertRaises(ValueError):
                    collect.prepare_proof(args, "init_namespace", row, root, {**identity, "input_identity": "wrong"})
                (root / "preparation/init_namespace/one/runner.json").write_text("changed")
                with self.assertRaises(ValueError):
                    collect.prepare_proof(args, "init_namespace", row, root, identity)

    def test_failed_preparation_never_starts_proof(self):
        row = {"scenario_id": "one"}
        args = SimpleNamespace(proof_preparation_spec={"cases": ["one"]})
        identity = {"source_identity": "source", "input_identity": "input"}
        with tempfile.TemporaryDirectory() as directory, patch.object(collect, "_run") as run, patch.object(
                collect, "prepare_proof", return_value={"status": "INCOMPLETE", "error": "preparation failed"}):
            result = collect.verify_row(args, "init_namespace", row, Path(directory), identity)
            self.assertEqual(result["status"], "INCOMPLETE")
            self.assertEqual(result["independent_preparation"]["status"], "INCOMPLETE")
            run.assert_not_called()

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
