"""Product-free checks for exact selected access receipts and case contract."""
import copy
import unittest
import historical_access as access


class AccessContract(unittest.TestCase):
    def test_oracle_and_cardinality_fail_closed(self):
        cases = access.definition()['cases']
        self.assertEqual([c['full157_index'] for c in cases], [1,157,1,157,1,157,58,58,157,157])
        for case in cases:
            output = 'storage_smoke_mount=fuse\naccess_completed_count=1\naccess_operation_ns=1\naccess_returned_bytes=' + str(case['length']) + '\n'
            output += ''.join('access_' + k + '=' + str(v) + '\n' for k,v in case['expected'].items())
            record = {'kind':'storage-smoke-execution', 'output':output}
            records = [copy.deepcopy(record) for _ in range(2 if case['cache']=='warm' else 1)]
            records += [{'kind':'historical-access-closed','cleanup_ok':True,'success':True},
                        {'kind':'storage-smoke-phase','phase':'access-measured','success':True}]
            access.check_output(records, case)
            for broken in (records[:-1], records + [record],
                           [{**record, 'output':output.replace('access_completed_count=1','access_completed_count=0')}] + records[1:]):
                with self.assertRaises(AssertionError): access.check_output(broken, case)
            broken = copy.deepcopy(records)
            key, value = next(iter(case['expected'].items()))
            broken[0]['output'] = output.replace('access_' + key + '=' + str(value), 'access_' + key + '=wrong')
            with self.assertRaises(AssertionError): access.check_output(broken, case)


if __name__ == '__main__': unittest.main()
