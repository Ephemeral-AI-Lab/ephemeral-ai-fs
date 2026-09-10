"""Offline boundaries for the #104 dependency-copy recipe in qualified builds."""
import fcntl
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import runner


class BuildReuseTests(unittest.TestCase):
    def test_image_archive_stays_under_runner_lock(self):
        with tempfile.TemporaryDirectory() as folder, patch.dict(os.environ, {'TMPDIR': folder}):
            def archive(tag):
                with (Path(folder) / 'layerfs-infra-measurement.lock').open('a') as lock:
                    with self.assertRaises(BlockingIOError):
                        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            with patch.object(runner, 'source_build_args', return_value={'LAYERFS_SOURCE_SEAL': 'a' * 64}), \
                    patch.object(runner.runtime, 'build_image', return_value=SimpleNamespace(returncode=0)), \
                    patch.object(runner, 'archive_image', side_effect=archive) as archived:
                self.assertEqual(runner.main(['--build-image']), 0)
                archived.assert_called_once()

    def test_compilation_and_dependency_invalidation(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            bench = root / 'benchmark/fs-bench-pro'
            paths = ['Cargo.toml', 'Cargo.lock', '.dockerignore', 'crates/store/src/lib.rs',
                     'crates/store/build.rs', 'crates/store/sql/schema.sql',
                     'benchmark/fs-bench-pro/Cargo.toml',
                     'benchmark/fs-bench-pro/Dockerfile.layerfs',
                     'benchmark/fs-bench-pro/src/main.rs',
                     'benchmark/fs-bench-pro/families/example.rs',
                     'benchmark/fs-bench-pro/workload/main.rs',
                     'benchmark/fs-bench-pro/build.rs', '.cargo/config.toml']
            for name in paths:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(name)
            with patch.object(runner, 'REPO', root), patch.object(runner, 'BENCH', bench), patch.object(
                    runner.runtime, 'run', return_value=SimpleNamespace(stdout=b'toolchain-1')) as command:
                def seals():
                    return runner.compilation_seal(), runner.compilation_seal(dependencies_only=True)
                baseline = seals()
                for name in paths:
                    path = root / name
                    original = path.read_bytes()
                    path.write_bytes(original + b' changed')
                    changed = seals()
                    self.assertNotEqual(changed[0], baseline[0], name)
                    benchmark_source = name in ('benchmark/fs-bench-pro/src/main.rs',
                                                'benchmark/fs-bench-pro/families/example.rs')
                    self.assertEqual(changed[1] == baseline[1], benchmark_source, name)
                    path.write_bytes(original)
                for name in ('docs.md', 'benchmark/fs-bench-pro/families/test.py'):
                    (root / name).write_text('non-native change')
                    self.assertEqual(seals(), baseline)
                for env in ('RUSTFLAGS', 'CARGO_BUILD_TARGET', 'CARGO_PROFILE_RELEASE_LTO',
                            'SDKROOT', 'CC', 'CARGO_ENCODED_RUSTFLAGS'):
                    with patch.dict(os.environ, {env: 'changed'}):
                        self.assertTrue(all(a != b for a, b in zip(seals(), baseline)), env)
                command.return_value = SimpleNamespace(stdout=b'toolchain-2')
                self.assertTrue(all(a != b for a, b in zip(seals(), baseline)))

    def test_seed_is_independent_excludes_benchmark_and_fails_closed(self):
        with tempfile.TemporaryDirectory() as folder, patch.object(runner, 'HOST_ROOT', Path(folder)):
            root = Path(folder)
            binary = root / 'fs-benchmark-pro'
            binary.write_bytes(b'qualified')
            source = root / 'builds/native-old'
            for name in ('release/.fingerprint/fs-benchmark-pro-123/bin',
                         'release/deps/fs_benchmark_pro-123.d', 'release/fs-benchmark-pro',
                         'release/deps/libstore.rlib', 'release/.fingerprint/store-123/lib'):
                path = source / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b'compiled')
            previous = {'LAYERFS_COMPILATION_SEAL': 'old', 'LAYERFS_DEPENDENCY_SEAL': 'deps',
                        'build_target': str(source), 'binary_sha256': runner.runtime.file_sha256(binary)}
            Path(str(binary) + '.identity.json').write_text(json.dumps(previous))
            target = root / 'builds/native-new'
            self.assertIsNone(runner.seed_host_dependencies(target, {'LAYERFS_DEPENDENCY_SEAL': 'other'}, binary))
            self.assertFalse(target.exists())
            values = {'LAYERFS_DEPENDENCY_SEAL': 'deps'}
            receipt = runner.seed_host_dependencies(target, values, binary)
            self.assertTrue(receipt['independent_copy'])
            self.assertFalse(any('fs_benchmark_pro' in str(p) or 'fs-benchmark-pro' in str(p)
                                 for p in target.rglob('*')))
            copied = target / 'release/deps/libstore.rlib'
            copied.write_bytes(b'new candidate')
            self.assertEqual((source / 'release/deps/libstore.rlib').read_bytes(), b'compiled')
            self.assertIsNone(runner.seed_host_dependencies(target, values, binary))
            binary.write_bytes(b'corrupt')
            with self.assertRaisesRegex(ValueError, 'producer binary identity'):
                runner.seed_host_dependencies(root / 'builds/native-third', values, binary)


if __name__ == '__main__':
    unittest.main()
