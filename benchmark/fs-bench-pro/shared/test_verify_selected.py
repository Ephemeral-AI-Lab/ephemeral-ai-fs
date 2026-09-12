import argparse
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
import runner as shared_runner


SCRIPT = Path(__file__).resolve().parent.parent / "verify-selected.py"
SPEC = importlib.util.spec_from_file_location("verify_selected", SCRIPT)
VERIFY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFY)


class Clock:
    def __init__(self, value=100.0):
        self.value = value

    def __call__(self):
        return self.value


def selected(**changes):
    value = {
        "family": "payload_create_read",
        "case": "payload-create-1m",
        "seed": 1,
        "repetition": None,
        "source_identity": "source-sha256",
        "input_identity": "input-sha256",
        "setup_identity": "clone",
        "product_identity": "product-sha256",
        "harness_identity": "harness-sha256",
        "image_identity": "image-sha256",
        "environment_identity": "environment-sha256",
        "verification_supported": True,
    }
    value.update(changes)
    return value


class VerifySelectedTests(unittest.TestCase):
    def test_selection_requires_exact_registry_and_content_identities(self):
        args = argparse.Namespace(family="payload_create_read", case="payload-create-1m")
        self.assertEqual(VERIFY.validate_selection(args, selected())["input_identity"], "input-sha256")
        with self.assertRaisesRegex(ValueError, "input_identity"):
            VERIFY.validate_selection(args, selected(input_identity=None))
        args.case = "all"
        with self.assertRaisesRegex(ValueError, "exactly one"):
            VERIFY.validate_selection(args, selected())

    def test_cleanup_failure_cannot_pass(self):
        result = VERIFY.normalize_result({"status": "PASS", "cleanup": {"status": "FAIL"}})
        self.assertEqual(result["status"], "INCOMPLETE")

    def test_live_runner_measurements_remain_bounded_receipt_fields(self):
        fields = VERIFY._bounded_result_fields({
            "phase": "resource-finalization",
            "resources": {"command_window_cpu_ns": 7},
            "setup": {"setup_mode": "fresh-output", "manifest_sha256": "abc"},
            "preparation_wall_ns": 11, "command_wall_ns": 13,
        })
        self.assertEqual(fields["phase"], "resource-finalization")
        self.assertEqual(fields["resources"]["command_window_cpu_ns"], 7)
        self.assertEqual(fields["setup_observation"]["setup_mode"], "fresh-output")
        self.assertEqual(fields["preparation_wall_ns"], 11)
        self.assertEqual(fields["command_wall_ns"], 13)

    def test_reuse_requires_every_identity_and_prior_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "verification.json"
            prior = {
                "schema": "layerfs-selected-verification-v2", "status": "PASS",
                "cleanup": {"status": "PASS"}, "wall_seconds": 12.0, **selected(),
            }
            path.write_text(json.dumps(prior))
            self.assertEqual(VERIFY.reuse_pass(path, selected())["status"], "PASS")
            with self.assertRaisesRegex(ValueError, "does not exactly match"):
                VERIFY.reuse_pass(path, selected(input_identity="different"))
            with self.assertRaisesRegex(ValueError, "does not exactly match"):
                VERIFY.reuse_pass(path, selected(sequence={"edit_count": 100}))

    def test_scaled_reuse_binds_policy_and_actual_hard_limit(self):
        selection = selected(family="init_namespace", case="namespace-100000",
            sequence={"schema": "workspace-sequence-v1", "edit_count": 1000, "commits": 33})
        selection["verification_policy"] = shared_runner.verification_policy(selection)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "verification.json"
            prior = {"schema": "layerfs-selected-verification-v2", "status": "PASS",
                "cleanup": {"status": "PASS"}, "wall_seconds": 500, "hard_limit_seconds": 614, **selection}
            path.write_text(json.dumps(prior))
            self.assertEqual(VERIFY.reuse_pass(path, selection)["status"], "PASS")
            for change in ({"hard_limit_seconds": 615}, {"wall_seconds": 614},
                           {"verification_policy": shared_runner.verification_policy({})}):
                path.write_text(json.dumps({**prior, **change}))
                with self.assertRaisesRegex(ValueError, "does not exactly match"):
                    VERIFY.reuse_pass(path, selection)

    def test_scaled_deadline_uses_original_start_and_keeps_selection_bounded(self):
        for count, commits, selection_wall, execution_offset, work, hard, status in (
                (100, 1, 30, -1, 45, 59, "PASS"),
                (1000, 33, 30, -1, 600, 614, "PASS"),
                (1000, 33, 30, 0, 600, 614, "TIMEOUT"),
                (1000, 33, 45, -1, 45, 59, "TIMEOUT")):
            with self.subTest(count=count, commits=commits, selection_wall=selection_wall), \
                 tempfile.TemporaryDirectory() as directory, mock.patch.dict("os.environ", {"TMPDIR": directory}):
                clock = Clock()
                class Runner:
                    build_parser = staticmethod(shared_runner.build_parser)
                    verification_policy = staticmethod(shared_runner.verification_policy)
                    execution_deadline = None

                    @staticmethod
                    def resolve_selection(args, deadline):
                        self.assertEqual(deadline, 145)
                        clock.value += selection_wall
                        return selected(family=args.family, case=args.case,
                            sequence={"schema": "workspace-sequence-v1", "edit_count": count, "commits": commits})

                    @staticmethod
                    def execute_selected(args, deadline, verification):
                        Runner.execution_deadline = deadline
                        clock.value = deadline + execution_offset
                        return {"status": "PASS", "cleanup": {"status": "PASS"}}

                output = Path(directory) / "run"
                argv = ["--family", "init_namespace", "--case", "namespace-100000", "--sequence", str(count),
                    "--sequence-commits", str(commits), "--seed", "1", "--image", "image-sha256",
                    "--source", "source-sha256", "--input", "input-sha256", "--output", str(output),
                    "--timeout", "999", "--product-timeout", "998"]
                self.assertEqual(VERIFY.run(Runner, argv, clock=clock), 0 if status == "PASS" else 1)
                receipt = json.loads((output / "verification.json").read_text())
                self.assertEqual(receipt["status"], status)
                self.assertEqual(receipt["monotonic_start_seconds"], 100)
                self.assertEqual(receipt["work_deadline_seconds"], 100 + work)
                self.assertEqual(receipt["hard_limit_seconds"], hard)
                self.assertEqual(Runner.execution_deadline, 100 + work if selection_wall < 45 else None)

    def test_scaled_publication_cannot_leave_late_pass(self):
        for delayed_final_link in (False, True):
            with self.subTest(delayed_final_link=delayed_final_link), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / "run"; output.mkdir()
                clock = Clock()
                receipt = {"status": "PASS", "error": None, "hard_limit_seconds": 614,
                    "monotonic_start_seconds": clock(), "_hard_deadline": clock() + 614}
                calls = 0
                def writer(path, data):
                    VERIFY._write(path, data)
                    if not delayed_final_link: clock.value += 614
                def linker(source, destination):
                    nonlocal calls
                    calls += 1; VERIFY.os.link(source, destination)
                    if delayed_final_link and calls == 2: clock.value += 614
                VERIFY.publish_receipt(output, receipt, clock=clock, stage_writer=writer, linker=linker)
                written = json.loads((output / "verification.json").read_text())
                self.assertEqual(written["status"], "TIMEOUT")
                self.assertIn("614-second", written["error"])

    def test_delayed_publication_cannot_leave_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "run"
            output.mkdir()
            clock = Clock()
            receipt = {
                "status": "PASS", "error": None,
                "monotonic_start_seconds": clock(), "_hard_deadline": clock() + 59.0,
            }

            def delayed_writer(path, data):
                VERIFY._write(path, data)
                clock.value += 59.0

            VERIFY.publish_receipt(output, receipt, clock=clock, stage_writer=delayed_writer)
            written = json.loads((output / "verification.json").read_text())
            self.assertEqual(written["status"], "TIMEOUT")

    def test_delayed_final_link_cannot_leave_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "run"
            output.mkdir()
            clock = Clock()
            receipt = {
                "status": "PASS", "error": None,
                "monotonic_start_seconds": clock(), "_hard_deadline": clock() + 59.0,
            }

            calls = 0

            def delayed_link(source, destination):
                nonlocal calls
                calls += 1
                VERIFY.os.link(source, destination)
                if calls == 2:
                    clock.value += 59.0

            VERIFY.publish_receipt(output, receipt, clock=clock, linker=delayed_link)
            written = json.loads((output / "verification.json").read_text())
            self.assertEqual(written["status"], "TIMEOUT")

    def test_failure_log_is_bounded_and_redacted(self):
        value = "token=visible " + "x" * (VERIFY.FAILURE_LOG_LIMIT + 100)
        encoded = VERIFY._sanitized_failure(value)
        self.assertLessEqual(len(encoded), VERIFY.FAILURE_LOG_LIMIT)
        self.assertNotIn(b"visible", encoded)
        self.assertIn(b"truncated", encoded)

    def test_registry_unsupported_stops_before_execution_and_keeps_compact_output(self):
        class Runner:
            executed = False
            verification_policy = staticmethod(shared_runner.verification_policy)

            @staticmethod
            def build_parser(include_modes=True):
                parser = argparse.ArgumentParser()
                parser.add_argument("--family", required=True)
                parser.add_argument("--case")
                parser.add_argument("--seed", type=int)
                parser.add_argument("--repetition", type=int)
                parser.add_argument("--setup")
                parser.add_argument("--verification", action="store_true")
                parser.add_argument("--perf-fast", action="store_true")
                parser.add_argument("--perf-samples", type=int)
                parser.add_argument("--smoke", action="store_true")
                parser.add_argument("--list", action="store_true")
                parser.add_argument("--prepare-only", action="store_true")
                parser.add_argument("--image")
                parser.add_argument("--source")
                parser.add_argument("--input")
                parser.add_argument("--output")
                return parser

            @staticmethod
            def resolve_selection(args, deadline):
                return selected(
                    family=args.family, case=args.case, seed=args.seed,
                    verification_supported=False, unsupported_reason="semantic proof exceeds 59 seconds",
                )

            @staticmethod
            def execute_selected(args, deadline, verification):
                Runner.executed = True
                raise AssertionError("unsupported proof executed")

        with tempfile.TemporaryDirectory() as directory, mock.patch.dict("os.environ", {"TMPDIR": directory}):
            output = Path(directory) / "run"
            argv = [
                "--family", "payload_create_read", "--case", "payload-create-1m",
                "--seed", "1", "--image", "image-sha256", "--source", "source-sha256",
                "--input", "input-sha256", "--output", str(output),
            ]
            self.assertEqual(VERIFY.run(Runner, argv), 1)
            self.assertFalse(Runner.executed)
            self.assertEqual({path.name for path in output.iterdir()}, {"verification.json", "failure.log"})
            self.assertEqual(json.loads((output / "verification.json").read_text())["status"], "INCOMPLETE")


if __name__ == "__main__":
    unittest.main()
