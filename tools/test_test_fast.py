import json
import os
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace

from test_fast import (DEFAULT_SECONDS, MAX_BATCH_TESTS, TIMINGS, load_timings, main,
                       plan_batches, run, seconds_for, split_binary)


class TestFastChecks(unittest.TestCase):
    def test_split_binary_covers_every_test_once_within_the_batch_cap(self):
        timings = {"module::test_7": 4.0, "module::test_9": 2.0}
        for count in (0, 1, 11, 12, 13, 159, 1000):
            names = [f"module::test_{index}" for index in range(count)]
            batches = split_binary(names, timings)
            if not names:
                self.assertEqual(batches, [])
                continue
            self.assertCountEqual([name for batch in batches for name in batch], names)
            self.assertEqual(len({name for batch in batches for name in batch}), count)
            self.assertTrue(all(1 <= len(batch) <= MAX_BATCH_TESTS for batch in batches))
            self.assertEqual(batches, split_binary(names, timings))

    def test_split_binary_separates_heavy_tests_and_keeps_every_test_once(self):
        names = ["light_a", "light_b", "light_c", "heavy"]
        batches = split_binary(names, {"heavy": 25.0})
        self.assertEqual([batch for batch in batches if "heavy" in batch], [["heavy"]])
        self.assertCountEqual([name for batch in batches for name in batch], names)

    def test_plan_is_longest_first_and_covers_every_test(self):
        discovered = [("binary-a", ".", [f"a::{index}" for index in range(30)]),
                      ("binary-b", ".", ["b::slow", "b::fast"])]
        timings = {"b::slow": 25.0, "a::0": 3.0}
        plan = plan_batches(discovered, timings)
        estimates = [sum(seconds_for(name, timings) for name in batch) for _, _, batch in plan]
        self.assertEqual(estimates, sorted(estimates, reverse=True))
        self.assertCountEqual([name for _, _, batch in plan for name in batch],
                              [name for _, _, names in discovered for name in names])
        self.assertEqual(plan[0], ("binary-b", ".", ["b::slow"]))

    def test_missing_or_damaged_timings_only_cost_ordering(self):
        self.assertEqual(load_timings("/nonexistent/timings.json"), {})
        with tempfile.TemporaryDirectory() as directory:
            for content in ("", "{", '{"tests": 3}', '{"tests": {"a": "slow"}}'):
                path = os.path.join(directory, "timings.json")
                with open(path, "w", encoding="utf-8") as sink:
                    sink.write(content)
                self.assertEqual(load_timings(path), {})
            path = os.path.join(directory, "timings.json")
            with open(path, "w", encoding="utf-8") as sink:
                json.dump({"tests": {"a": 1.5, "b": 0}}, sink)
            self.assertEqual(load_timings(path), {"a": 1.5, "b": 0.0})
        self.assertEqual(seconds_for("unmeasured", {}), DEFAULT_SECONDS)

    def test_committed_timings_are_measured_hints(self):
        timings = load_timings(TIMINGS)
        self.assertGreater(len(timings), 100)
        self.assertTrue(all(seconds >= 0 for seconds in timings.values()))
        self.assertTrue(all(name and isinstance(name, str) for name in timings))

    def test_invalid_jobs_cannot_skip_execution(self):
        for jobs in (0, -1, 17):
            with self.assertRaises(SystemExit):
                main("unused", jobs)

    def test_exact_selection_count_and_failure_are_enforced(self):
        item = ('test-binary', '.', ['module::test', 'module::test_long'])
        for code, output, expected in (
            (0, 'test result: ok. 1 passed; 0 failed; 1 ignored;', 0),
            (0, 'test result: ok. 0 passed; 0 failed; 0 ignored;', 1),
            (101, 'test result: FAILED. 1 passed; 1 failed; 0 ignored;', 101),
        ):
            with patch('test_fast.subprocess.run', return_value=SimpleNamespace(returncode=code, stdout=output)) as command:
                self.assertEqual(run(item)[1], expected)
                self.assertEqual(command.call_args.args[0],
                                 ['test-binary', '--test-threads=1', '--exact', *item[2]])


if __name__ == '__main__':
    unittest.main()
