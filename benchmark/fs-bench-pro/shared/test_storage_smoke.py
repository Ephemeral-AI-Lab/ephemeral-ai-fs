import copy
from types import SimpleNamespace
import unittest
from unittest import mock

import storage_smoke as storage


class HistoricalExecutionIdentityTests(unittest.TestCase):
    def setUp(self):
        self.current={'LAYERFS_SOURCE_SEAL':'current-source','LAYERFS_PRODUCT_SEAL':'current-product',
            'LAYERFS_COMPILATION_SEAL':'current-compiled','LAYERFS_SOURCE_COMMIT':'current-commit',
            'WORKLOAD_SOURCE_SHA256':'same-importer'}
        self.host={**self.current,'binary_sha256':'binary',
            'integrated_format_probe':{'status':'PASS','storage_policy':'ordinary','schema_version':10}}
        self.args=SimpleNamespace(host_binary='unused-test-binary',source_arm='candidate')
        patch=mock.patch.object(storage.runtime,'file_sha256',return_value='binary')
        patch.start(); self.addCleanup(patch.stop)

    def image(self,host):
        return {'Config':{'Labels':{'dev.layerfs.'+label:host[key] for key,label in
            [('LAYERFS_SOURCE_SEAL','source-seal'),('LAYERFS_PRODUCT_SEAL','product-seal'),
             ('LAYERFS_COMPILATION_SEAL','compilation-seal')]}}}

    def test_candidate_uses_current_exact_product_and_compilation(self):
        self.assertEqual(storage.validate_execution_identity(self.args,self.current,self.host,self.image(self.host)),self.current)

    def test_explicit_baseline_records_its_actual_producer_and_current_importer(self):
        old={**self.host,'LAYERFS_SOURCE_SEAL':'old-source','LAYERFS_PRODUCT_SEAL':'old-product',
            'LAYERFS_COMPILATION_SEAL':'old-compiled','LAYERFS_SOURCE_COMMIT':'old-commit'}
        with self.assertRaisesRegex(ValueError,'stale candidate'):
            storage.validate_execution_identity(self.args,self.current,old,self.image(old))
        self.args.source_arm='baseline'
        producer=storage.validate_execution_identity(self.args,self.current,old,self.image(old))
        self.assertEqual(producer['LAYERFS_SOURCE_COMMIT'],'old-commit')
        self.assertEqual(producer['LAYERFS_PRODUCT_SEAL'],'old-product')
        self.assertEqual(producer['WORKLOAD_SOURCE_SHA256'],self.current['WORKLOAD_SOURCE_SHA256'])

    def test_baseline_cannot_bypass_binary_image_workload_or_format_identity(self):
        self.args.source_arm='baseline'
        for mutation in ('binary','source-seal','product-seal','compilation-seal','workload','probe'):
            with self.subTest(mutation=mutation):
                host=copy.deepcopy(self.host); image=self.image(host)
                if mutation=='binary': host['binary_sha256']='changed'
                elif mutation=='workload': host['WORKLOAD_SOURCE_SHA256']='different-importer'
                elif mutation=='probe': host['integrated_format_probe']['storage_policy']='compacted'
                else: image['Config']['Labels']['dev.layerfs.'+mutation]='mismatched'
                with self.assertRaises(ValueError):
                    storage.validate_execution_identity(self.args,self.current,host,image)


if __name__=='__main__':unittest.main()
