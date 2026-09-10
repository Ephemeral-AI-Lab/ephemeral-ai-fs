import json
from pathlib import Path
import tempfile
import unittest

import integrated_storage as integrated

class IntegratedStorageTests(unittest.TestCase):
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
