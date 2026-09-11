"""Cold eligibility is mandatory even when timings and saved PASS look good."""
import copy
import json
import hashlib
import os
from pathlib import Path
import platform
import sys
import tempfile
import unittest
from unittest.mock import patch

import cold
import runner


def sample(arm="candidate", elapsed=2_600_000_000):
    start = 10_000_000_000 if arm == "baseline" else 20_000_000_000
    return {
        "status": "PASS", "qualification_eligible": True, "completion_status": "COMPLETE",
        "identities": {"family": "init_namespace", "case": "namespace-100000", "seed": 1,
            "source_arm": arm, "timer": "layerstack_init_ns", "harness_identity": runner.harness_identity(),
            "source_identity": "a" * 64, "product_identity": "b" * 64,
            "topology": "host-store", "environment": {"cpus": 2}, "host_environment": {"os": "Darwin"},
            "image": "image", "host_executor": {"WORKLOAD_SOURCE_SHA256": "w" * 64,
                "schema_sha256": "s" * 64, "rust_toolchain": "1.85.1", "binary_sha256": "c" * 64}},
        "setup": {"sample_root": arm}, "cleanup": {"status": "PASS"},
        "resources": {"oom_kill_delta": 0, "swap_current_bytes": 0},
        "product_command_started_ns": start,
        "cold_acquisition": {"contract": cold.CONTRACT, "method": cold.METHOD,
            "status": "VERIFIED_COLD", "sample_root": arm, "finished_ns": start - 1,
            "fixture_digest": cold.FIXTURE_DIGEST, "fixture_manifest_sha256": "f" * 64,
            "files_checked": 100_000, "logical_bytes": 500_000_000,
            "allocated_bytes": 865_730_560, "pages_checked": 110_000, "expected_pages": 110_000,
            "resident_pages": 0, "errors": [],
            "metadata_validation": {"contract": "namespace-fixture-metadata-v1",
                "status": "VERIFIED", "files_checked": 100_000, "directories_checked": 1001,
                "file_mode": 0o640, "directory_mode": 0o750, "mtime_ns": 1_700_000_000_000_000_000},
            "backend_self_check": {"warm_pages_detected": 1, "cold_pages_remaining": 0}},
        "records": [{"layerstack_init_ns": elapsed, "fixture_digest": cold.FIXTURE_DIGEST,
            "fixture_cache_profile": "reused-first-sample-uncontrolled", "regular_files": 100_000,
            "scanned_files": 100_000, "logical_bytes": 500_000_000, "scanned_bytes": 500_000_000,
            "initialization_disk_read_bytes": 865_730_560, "process_t0_swaps": 0, "process_t1_swaps": 0}],
    }


class ColdTests(unittest.TestCase):
    def test_warm_26_seconds_cannot_pass_even_with_saved_pass_or_large_reads(self):
        for reads in (0, 900_000_000):
            row = sample()
            row["cold_acquisition"]["resident_pages"] = 1
            row["records"][0]["initialization_disk_read_bytes"] = reads
            original = copy.deepcopy(row["records"])
            summary = runner.performance_summary([row], 1, True)
            self.assertEqual(summary["status"], "INCOMPLETE")
            self.assertEqual(summary["valid"], 0)
            self.assertEqual(summary["diagnostic_samples"], 1)
            self.assertIsNone(summary["median_ns"])
            self.assertEqual(row["status"], "INELIGIBLE")
            self.assertEqual(row["records"], original)

    def test_missing_incomplete_stale_or_foreign_cache_receipt_fails_closed(self):
        mutations = [lambda r: r.pop("cold_acquisition"),
            lambda r: r["cold_acquisition"].update(status="UNVERIFIED"),
            lambda r: r["cold_acquisition"].update(files_checked=99_999),
            lambda r: r["cold_acquisition"].update(pages_checked=109_999),
            lambda r: r["cold_acquisition"].update(sample_root="another-run"),
            lambda r: r["cold_acquisition"].update(finished_ns=0),
            lambda r: r["cold_acquisition"].update(backend_self_check={}),
            lambda r: r["records"][0].update(initialization_disk_read_bytes=0),
            lambda r: r["records"][0].update(fixture_cache_profile="reused-subsequent-sample-uncontrolled"),
            lambda r: r.update(cold_diagnostic_environment=True),
            lambda r: r["records"].append({"kind": "initialization-debug-text"}),
            lambda r: r["identities"].update(case="namespace-100000-text-v1"),
            lambda r: r["identities"].update(timer="another_timer"),
            lambda r: r["cleanup"].update(status="FAIL")]
        for mutate in mutations:
            row = sample()
            mutate(row)
            with self.subTest(mutate=mutate):
                self.assertFalse(cold.assess(row)["qualification_eligible"])

    def test_cold_limit_cannot_be_waived_by_collection_mode(self):
        for elapsed, expected in ((cold.TARGET_NS, "PASS"), (cold.TARGET_NS + 1, "TARGET_MISS")):
            row = sample(elapsed=elapsed)
            result = runner.performance_summary([row], 1, True)
            self.assertEqual(result["status"], expected)
            self.assertEqual(result["product_target_ns"], cold.TARGET_NS)
            self.assertEqual(result["median_ns"], elapsed)
            self.assertFalse(result["admission_eligible"])

    def test_mixed_pair_and_identity_mismatches_have_no_speedup(self):
        control = sample("baseline", 3_700_000_000)
        for change in (lambda r: r.pop("cold_acquisition"),
                       lambda r: r["cold_acquisition"].update(resident_pages=10),
                       lambda r: r["cold_acquisition"].update(method="warm-up"),
                       lambda r: r["identities"].update(seed=2),
                       lambda r: r["identities"].update(harness_identity="different"),
                       lambda r: r["identities"]["host_executor"].update(WORKLOAD_SOURCE_SHA256="different")):
            candidate = sample()
            change(candidate)
            result = cold.compare(control, candidate, order=["baseline", "candidate"])
            self.assertEqual(result["status"], "INELIGIBLE")
            self.assertIsNone(result["reduction_ns"])
            self.assertIsNone(result["reduction_percent"])

    def test_compatible_cold_pair_and_observed_order(self):
        control, candidate = sample("baseline", 3_700_000_000), sample()
        result = cold.compare(control, candidate, order=["baseline", "candidate"])
        self.assertEqual(result["reduction_ns"], 1_100_000_000)
        self.assertAlmostEqual(result["reduction_percent"], 100 * 1.1 / 3.7)
        self.assertIsNone(cold.compare(control, candidate, order=["candidate", "baseline"])["reduction_ns"])

    def test_collector_rechecks_raw_sample_instead_of_forged_summary(self):
        sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
        import issue54_collect
        row = sample()
        row["kind"] = "sample"
        row.pop("cold_acquisition")
        with tempfile.TemporaryDirectory() as directory:
            receipt = Path(directory) / "perf.jsonl"
            receipt.write_text(json.dumps(row) + '\n' + json.dumps({"kind": "summary", "status": "PASS"}) + '\n')
            result = issue54_collect.load_performance(receipt, "init_namespace",
                {"scenario_id": "namespace-100000", "route": "namespace"}, "a" * 64, "image")
            self.assertEqual(result["status"], "INELIGIBLE")
            self.assertIsNone(result["elapsed_ns"])
            self.assertEqual(result["diagnostic_elapsed_ns"], 2_600_000_000)

    def test_missing_partial_or_wrong_metadata_evidence_never_qualifies(self):
        mutations = [lambda r: r["cold_acquisition"].pop("metadata_validation"),
            lambda r: r["cold_acquisition"].update(contract="namespace-100000-cold-v1"),
            lambda r: r["cold_acquisition"]["metadata_validation"].update(status="UNVERIFIED"),
            lambda r: r["cold_acquisition"]["metadata_validation"].update(files_checked=99_999),
            lambda r: r["cold_acquisition"]["metadata_validation"].update(directories_checked=1000),
            lambda r: r["cold_acquisition"]["metadata_validation"].update(directory_mode=0o755),
            lambda r: r["cold_acquisition"]["metadata_validation"].update(file_mode=0o644),
            lambda r: r["cold_acquisition"]["metadata_validation"].update(mtime_ns=0)]
        for mutate in mutations:
            row = sample()
            mutate(row)
            original = copy.deepcopy(row["records"])
            with self.subTest(mutate=mutate):
                result = runner.performance_summary([row], 1, True)
                self.assertEqual(result["valid"], 0)
                self.assertIsNone(result["median_ns"])
                self.assertEqual(row["records"], original)
                self.assertIsNone(cold.compare(sample("baseline"), row,
                    order=["baseline", "candidate"])["reduction_percent"])

    def fixture(self, root):
        directory = root / "payload" / "d0000"
        directory.mkdir(parents=True)
        file = directory / "file"
        file.write_bytes(b"abc")
        for path in (file, directory, root / "payload"):
            path.chmod(0o640 if path == file else 0o750)
            os.utime(path, ns=(1_700_000_000_000_000_000,) * 2)
        files = {"payload/d0000/file": {"bytes": 3, "sha256": hashlib.sha256(b"abc").hexdigest()}}
        (root / "host-cache.json").write_text(json.dumps({"files": files}))
        return files

    def test_unchanged_bytes_and_file_metadata_do_not_hide_directory_mtime(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            files = self.fixture(root)
            self.assertEqual(set(cold._metadata_inventory(root, files, lambda: None)),
                {root / "payload", root / "payload/d0000"})
            file = root / "payload/d0000/file"
            before = file.stat()
            # The cp -cR failure: file bytes/mode/mtime agree, directory mtime does not.
            os.utime(root / "payload/d0000", ns=(1_800_000_000_000_000_000,) * 2)
            self.assertEqual(hashlib.sha256(file.read_bytes()).hexdigest(), files["payload/d0000/file"]["sha256"])
            self.assertEqual((file.stat().st_mode, file.stat().st_mtime_ns),
                             (before.st_mode, before.st_mtime_ns))
            with self.assertRaisesRegex(ValueError, "metadata"):
                cold._metadata_inventory(root, files, lambda: None)
            with patch.object(cold, "Residency") as backend:
                backend.return_value.page_size = 4096
                result = cold.acquire({"host_root": str(root), "fixture": {}}, "sample", float("inf"))
            self.assertEqual(result["status"], "UNVERIFIED")
            self.assertIn("metadata mismatch", result["errors"][0])

    def test_root_directory_and_file_metadata_drift_are_rejected(self):
        for relative, mode, stamp in [("payload", None, 0),
                ("payload/d0000", 0o755, None), ("payload/d0000/file", 0o644, None),
                ("payload/d0000/file", None, 0)]:
            with self.subTest(relative=relative, mode=mode, stamp=stamp), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                files = self.fixture(root)
                if mode is not None:
                    (root / relative).chmod(mode)
                if stamp is not None:
                    os.utime(root / relative, ns=(stamp, stamp))
                with self.assertRaisesRegex(ValueError, "metadata"):
                    cold._metadata_inventory(root, files, lambda: None)

    def test_extra_empty_directory_and_symlink_root_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            files = self.fixture(root)
            extra = root / "payload/extra"
            extra.mkdir(mode=0o750)
            for path in (extra, root / "payload"):
                os.utime(path, ns=(1_700_000_000_000_000_000,) * 2)
            with self.assertRaisesRegex(ValueError, "inventory"):
                cold._metadata_inventory(root, files, lambda: None)
            (root / "payload").rename(root / "original")
            (root / "payload").symlink_to(root / "original", target_is_directory=True)
            with self.assertRaisesRegex(ValueError, "metadata"):
                cold._metadata_inventory(root, files, lambda: None)

    def test_unsupported_backend_is_retained_as_unverified(self):
        with patch.object(cold, "Residency", side_effect=OSError("unsupported")):
            result = cold.acquire({}, "sample", float("inf"))
        self.assertEqual(result["status"], "UNVERIFIED")
        self.assertEqual(result["errors"], ["unsupported"])

    @unittest.skipUnless(platform.system() == "Darwin", "Darwin residency backend")
    def test_native_backend_detects_warm_input_and_checks_invalidation(self):
        backend = cold.Residency()
        result = backend.self_check()
        self.assertGreater(result["warm_pages_detected"], 0)
        self.assertEqual(result["cold_pages_remaining"], 0)


if __name__ == "__main__":
    unittest.main()
