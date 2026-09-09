import unittest
from unittest.mock import patch
import repository_history as history


class HistorySelection(unittest.TestCase):
    def test_optional_exact_endpoints_and_explicit_dispatch(self):
        rows=history.registry()
        self.assertEqual([r['state_count'] for r in rows],[157,53,17])
        self.assertEqual(rows[-1]['full157_indices'],list(range(1,158,10))+[157])
        self.assertTrue(all(r['default_status']=='NOT_RUN_OPTIONAL' for r in rows))
        with self.assertRaises(SystemExit):history.main([])
        with patch('storage_smoke.main',return_value=0) as run:
            history.main(['--profile','stride-10','--output','example'])
            run.assert_called_once_with(['--storage-smoke','deepseek-stride10','--output','example'])


if __name__=='__main__':unittest.main()
