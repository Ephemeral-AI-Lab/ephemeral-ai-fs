import contextlib
import io
import json
from pathlib import Path
import tempfile
import unittest

import integrated_storage as integrated

class IntegratedStorageTests(unittest.TestCase):
    def test_removed_compaction_flag_is_rejected_before_build_or_run(self):
        import storage_smoke
        with contextlib.redirect_stderr(io.StringIO()) as stderr:
            with self.assertRaises(SystemExit) as error:
                storage_smoke.main(['--storage-smoke', 'deepseek-stride3', '--storage-compact'])
        self.assertEqual(error.exception.code, 2)
        self.assertIn('unrecognized arguments: --storage-compact', stderr.getvalue())

    def test_full_history_keeps_original_access_checkpoints_and_distinct_custody(self):
        template=json.loads((integrated.runner.BENCH/'families/historical_access/fixture.json').read_text())
        full=integrated.PROFILES['deepseek-full']; stride=integrated.PROFILES['deepseek-stride3']
        self.assertEqual(full['indices'],tuple(range(1,158)))
        self.assertEqual(stride['indices'],tuple(range(1,158,3)))
        mapped=lambda profile: [profile['checkpoint_map'].get(c['full157_index'],c['full157_index']) for c in template['cases']]
        self.assertEqual(mapped(full),[c['full157_index'] for c in template['cases']])
        self.assertEqual(set(mapped(full)),{1,57,65,157})
        self.assertEqual(set(mapped(stride)),{1,58,67,157})
        for profile in (full,stride):
            self.assertTrue(set(mapped(profile))<=set(profile['indices']))
            self.assertTrue((integrated.runner.REPO/profile['contract']).is_file())
        for field in ('contract','scenario','access_profile','case_suffix'):
            self.assertNotEqual(full[field],stride[field])
    def test_freeze_counts_complete_directory_and_rejects_sidecars(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);store=root/'store.sqlite';store.write_bytes(b'x'*12345)
            frozen=integrated.freeze(store)
            self.assertEqual(frozen['allocated_bytes'],store.stat().st_blocks*512)
            self.assertEqual(frozen['files']['store.sqlite']['inode'],store.stat().st_ino)
            self.assertEqual(frozen['apparent_bytes'],12345)
            (root/'store.sqlite-journal').write_bytes(b'x')
            with self.assertRaises(ValueError):integrated.freeze(store)
    def test_freeze_refuses_symlink_dependencies(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);store=root/'store.sqlite';store.write_bytes(b'x')
            (root/'adapter').symlink_to(store)
            with self.assertRaises(ValueError):integrated.freeze(store)
    def test_structured_receipts_cannot_silently_ignore_non_json(self):
        self.assertEqual(integrated.records(b'{"kind":"storage-compaction"}\n'),[{'kind':'storage-compaction'}])
        with self.assertRaises(json.JSONDecodeError):integrated.records(b'fallback reader passed\n')

if __name__=='__main__':unittest.main()
