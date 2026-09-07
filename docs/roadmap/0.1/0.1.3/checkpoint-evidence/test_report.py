"""A mismatched proof or absent sample must never become checkpoint PASS."""
import csv
import json
from pathlib import Path
import tempfile
import unittest
import report


class ReportTest(unittest.TestCase):
    def test_missing_mismatched_and_duplicate_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            family, case = 'git_tool_workflow', 'git-tool-100-mixed-v4'
            registry = [{'family_id': family, 'scenario_id': case, 'proof_only': False,
                         'verification_supported': True}]
            self.assertEqual(report.derive(root, registry)['status'], 'INCOMPLETE')
            identity = {'case': case, 'family': family, 'source_identity': 'source',
                        'product_identity': 'product', 'image': 'image', 'input_identity': 'input', 'harness_identity': 'harness'}
            sample = {'kind': 'sample', 'status': 'PASS', 'identities': identity,
                      'cleanup': {'status': 'PASS'}, 'prepared_master_unchanged': True,
                      'environment_observation': {'validated': True},
                      'resources': {'oom_kill_delta': 0, 'swap_current_bytes': 0}, 'records': [{'pure_call_sum_ns': 2_000_000_000}]}
            (root / 'declaration.json').write_text(json.dumps({
                'performance_order': [case], 'verification_order': [case], 'image': 'image',
                'harness_identity': 'harness',
                'host': {'LAYERFS_SOURCE_SEAL': 'source', 'LAYERFS_PRODUCT_SEAL': 'product'},
            }))
            performance = root / 'performance' / family / case / 'perf.jsonl'
            performance.parent.mkdir(parents=True)
            performance.write_text(json.dumps(sample) + '\n' + json.dumps({'kind': 'summary', 'status': 'PASS'}) + '\n')
            proof = root / 'verification' / family / case / 'verification.json'
            proof.parent.mkdir(parents=True)
            receipt = {**identity, 'image_identity': 'image', 'status': 'PASS', 'cleanup': {'status': 'PASS'}}
            proof.write_text(json.dumps(receipt))
            result = report.derive(root, registry)
            self.assertEqual(result['status'], 'PASS', result['errors'])
            self.assertEqual(result['performance'][0]['target_status'], 'TARGET_MISS')
            report.write(result, root / 'tables')
            with (root / 'tables/performance.csv').open() as stream:
                row = next(csv.DictReader(stream))
                self.assertEqual(row['source_identity'], 'source')
                self.assertEqual(row['cleanup_status'], 'PASS')
                self.assertIn('fixture_bytes', row)
            with (root / 'tables/verification.csv').open() as stream:
                self.assertEqual(next(csv.DictReader(stream))['status'], 'PASS')
            sample['records'][0]['resource_status'] = 'fail'
            performance.write_text(json.dumps(sample) + '\n' + json.dumps({'kind': 'summary', 'status': 'PASS'}) + '\n')
            self.assertEqual(report.derive(root, registry)['status'], 'INCOMPLETE')
            sample['records'][0].pop('resource_status')
            performance.write_text(json.dumps(sample) + '\n' + json.dumps({'kind': 'summary', 'status': 'PASS'}) + '\n')
            self.assertEqual(report.derive(root, registry)['status'], 'PASS')
            receipt['input_identity'] = 'different input'
            proof.write_text(json.dumps(receipt))
            self.assertEqual(report.derive(root, registry)['status'], 'INCOMPLETE')
            performance.write_text((json.dumps(sample) + '\n') * 2)
            self.assertEqual(report.derive(root, registry)['status'], 'INCOMPLETE')
            with self.assertRaises(ValueError):
                report.derive(root, registry * 2)

    def test_verifier_requalification_requires_same_recipe_fixture_and_product(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            family, case = 'edit_length_changing', 'example'
            definition = {'family_id': family, 'scenario_id': case, 'proof_only': False,
                          'verification_supported': True, 'route': 'sdk'}
            old = {'LAYERFS_SOURCE_SEAL': 'old', 'LAYERFS_PRODUCT_SEAL': 'product'}
            new = {**old, 'LAYERFS_SOURCE_SEAL': 'new'}
            inputs = {source: report.runner.digest({'family': family, 'case': case, 'seed': 1,
                                                   'source': source, 'recipe': definition})
                      for source in ('old', 'new')}
            declaration = {'host': old, 'image': 'image', 'harness_identity': 'harness',
                           'performance_order': [case], 'verification_order': [case]}
            (root / 'declaration.json').write_text(json.dumps(declaration))
            identity = {'family': family, 'case': case, 'source_identity': 'old',
                        'input_identity': inputs['old'], 'product_identity': 'product',
                        'image': 'image', 'harness_identity': 'harness'}
            sample = {'kind': 'sample', 'status': 'PASS', 'identities': identity,
                      'records': [{'edit_commit_ns': 1000, 'row_resource_status': 'pass'}],
                      'cleanup': {'status': 'PASS'}, 'prepared_master_unchanged': True,
                      'environment_observation': {'validated': True},
                      'resources': {'oom_kill_delta': 0, 'swap_current_bytes': 0}}
            perf = root / 'performance' / family / case / 'perf.jsonl'
            perf.parent.mkdir(parents=True)
            perf.write_text(json.dumps(sample) + '\n' + json.dumps({'kind': 'summary', 'status': 'PASS'}) + '\n')
            original = {**identity, 'image_identity': 'image', 'status': 'FAIL',
                        'cleanup': {'status': 'PASS'}, 'preparation': {'fixture': {'fixture_sha256': 'bytes'}}}
            path = root / 'verification' / family / case / 'verification.json'
            path.parent.mkdir(parents=True)
            path.write_text(json.dumps(original))
            replacement = {**original, 'source_identity': 'new', 'input_identity': inputs['new'], 'status': 'PASS'}
            path = root / 'replacement.json'
            path.write_text(json.dumps(replacement))
            manifest = {'old_host': old, 'new_host': new, 'harness_identity': 'harness',
                        'changed_execution_files': ['benchmark/fs-bench-pro/src/sdk_edit_verify.rs'],
                        'cases': {case: {'family': family, 'recipe': definition, 'receipt': 'replacement.json',
                                         'old_input_identity': inputs['old'], 'new_input_identity': inputs['new']}}}
            (root / 'sdk-requalification.json').write_text(json.dumps(manifest))
            result = report.derive(root, [definition])
            self.assertEqual(result['status'], 'PASS', result['errors'])
            self.assertEqual(result['verification'][0]['original_status'], 'FAIL')
            replacement['preparation'] = {'fixture': {'fixture_sha256': 'changed bytes'}}
            path.write_text(json.dumps(replacement))
            self.assertEqual(report.derive(root, [definition])['status'], 'INCOMPLETE')
            replacement['preparation'] = original['preparation']
            replacement['product_identity'] = 'different product'
            path.write_text(json.dumps(replacement))
            self.assertEqual(report.derive(root, [definition])['status'], 'INCOMPLETE')


if __name__ == '__main__':
    unittest.main()
