import unittest
from unittest.mock import patch
from types import SimpleNamespace

from test_fast import main, run, test_batches


class TestFastChecks(unittest.TestCase):
    def test_batches_preserve_every_test_exactly_once(self):
        for count in (0, 1, 31, 32, 33, 64, 135, 1000):
            names = [f"module::test_{index}" for index in range(count)]
            for jobs in (1, 4, 16):
                batches = test_batches(names, jobs)
                self.assertGreaterEqual(len(batches), 1)
                self.assertLessEqual(len(batches), jobs)
                self.assertCountEqual([name for batch in batches for name in batch], names)

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
