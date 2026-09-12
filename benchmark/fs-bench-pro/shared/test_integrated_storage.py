import contextlib
import hashlib
import io
import json
from pathlib import Path
import sqlite3
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

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


class OrdinaryAccessTests(unittest.TestCase):
    def setUp(self):
        self.directory=tempfile.TemporaryDirectory(); self.addCleanup(self.directory.cleanup)
        root=Path(self.directory.name); self.run=root/'run'; self.folder=self.run/'deepseek-full'
        host=self.folder/'host-runtime'; host.mkdir(parents=True)
        self.store=host/'store.sqlite'
        with contextlib.closing(sqlite3.connect(self.store)) as db, db: db.execute('CREATE TABLE content(value)')
        (host/'branch-id').write_text('measured-branch')
        self.body=b'original authenticated bytes'
        digest=hashlib.sha256(self.body).hexdigest()
        oid=hashlib.sha1(b'blob '+str(len(self.body)).encode()+b'\0'+self.body).hexdigest()
        oracle=root/'oracle.json'; self.write(oracle,{'file'.encode().hex():['100644',len(self.body),digest]})
        inputs=root/'input'; inputs.mkdir()
        (inputs/'manifest.tsv').write_text(f'100644\t{oid}\t{len(self.body)}\t'+b'file'.hex()+'\n')
        identity={'smoke':'deepseek-full','storage_compact':False,
            'full_run_contract_sha256':integrated.runtime.file_sha256(integrated.runner.REPO/integrated.ORDINARY_FULL['contract']),
            'source':{'LAYERFS_SOURCE_COMMIT':'test-current-source'}}
        self.write(self.run/'identity.json',identity)
        self.rows=[{'index':i,'full157_index':i,'identity':f'commit-{i}','commit_id':f'commit-{i}',
            'sha':f'source-{i}','oracle':str(oracle),'oracle_sha256':integrated.runtime.file_sha256(oracle),
            'input':str(inputs)} for i in range(1,158)]
        self.write(self.folder/'performance-result.json',{'status':'PASS','cleanup_status':'PASS','records':self.rows})
        self.write(self.run/'performance-summary.json',{'status':'PASS'})
        self.write(self.run/'performance-manifest.json',{'deepseek-full/host-runtime/store.sqlite':integrated.runtime.file_sha256(self.store)})
        bench=root/'bench'; fixture=bench/'families/historical_access/fixture.json'; fixture.parent.mkdir(parents=True)
        self.cases=[]
        for i in range(11):
            operation=('stat','directory','read')[i%3]
            expected={'names':b'file'.hex()} if operation=='directory' else {'mode':'100644','size':len(self.body),'mtime':1000000000,'mtime_nsec':0}
            if operation=='read': expected['sha256']=hashlib.sha256(self.body[2:7]).hexdigest()
            self.cases.append({'id':f'case-{i}-v2','full157_index':(1,57,65,157)[i%4],
                'operation':operation,'path':'.' if operation=='directory' else 'file',
                'offset':2 if operation=='read' else 0,'length':5 if operation=='read' else 0,
                'cache':'cold','expected':expected})
        self.write(fixture,{'schema':'historical-access-v2','cases':self.cases})
        patch=mock.patch.object(integrated.runner,'BENCH',bench); patch.start(); self.addCleanup(patch.stop)
        patch=mock.patch.object(integrated.runtime,'run',return_value=SimpleNamespace(stdout=self.body)); patch.start(); self.addCleanup(patch.stop)
        self.output=root/'access.json'

    def write(self,path,value):
        path.write_text(json.dumps(value))

    def verify(self):
        self.write(self.run/'verification-summary.json',{'status':'PASS'})
        self.write(self.folder/'verification-result.json',{'status':'PASS','cleanup_status':'PASS',
            'records':[{'index':r['index'],'identity':r['identity'],'status':'PASS'} for r in self.rows]})

    def test_ordinary_freeze_and_complete_verification_keep_measured_image_and_all_original_oracles(self):
        measured=integrated.runtime.file_sha256(self.store)
        with contextlib.redirect_stdout(io.StringIO()): integrated.freeze_access(self.run)
        # The real full-history verifier may publish bookkeeping after the measured window.
        with contextlib.closing(sqlite3.connect(self.store)) as db, db: db.execute("INSERT INTO content VALUES ('verification')")
        self.verify()
        with contextlib.redirect_stdout(io.StringIO()): integrated.prepare_access(self.run,self.output,self.run)
        fixture=json.loads(self.output.read_text())
        self.assertEqual(fixture['store_sha256'],measured)
        self.assertEqual(integrated.runtime.file_sha256(Path(fixture['store'])),measured)
        self.assertNotEqual(fixture['verification_store']['sha256'],measured)
        self.assertEqual(fixture['measured_store']['sha256'],measured)
        self.assertEqual(fixture['branch_id'],'measured-branch')
        self.assertEqual(fixture['profile'],'historical-access-full157-ordinary-v1')
        self.assertNotIn('compaction_result_sha256',fixture)
        self.assertEqual(len(fixture['cases']),11)
        self.assertEqual([c['expected'] for c in fixture['cases']],[c['expected'] for c in self.cases])
        self.assertEqual([c['full157_index'] for c in fixture['cases']],[c['full157_index'] for c in self.cases])
        self.assertTrue(all(c['commit_id']==f"commit-{c['full157_index']}" for c in fixture['cases']))

    def test_cannot_freeze_after_verification_begins(self):
        (self.folder/'verification-pending-1.json').write_text('{}')
        with self.assertRaisesRegex(ValueError,'before verification starts'): integrated.freeze_access(self.run)
        self.assertFalse((self.folder/'frozen-measured-store').exists())

    def test_changed_measured_store_rejected_before_copy(self):
        with contextlib.closing(sqlite3.connect(self.store)) as db, db: db.execute("INSERT INTO content VALUES ('changed')")
        with self.assertRaisesRegex(ValueError,'changed before freeze'): integrated.freeze_access(self.run)
        self.assertFalse((self.folder/'frozen-measured-store').exists())

    def test_all_157_verification_identities_and_cleanup_are_required(self):
        with contextlib.redirect_stdout(io.StringIO()): integrated.freeze_access(self.run)
        self.verify()
        path=self.folder/'verification-result.json'; complete=json.loads(path.read_text())
        for mutation in ('missing','wrong-identity','cleanup'):
            with self.subTest(mutation=mutation):
                value=json.loads(json.dumps(complete))
                if mutation=='missing': value['records'].pop(120)
                elif mutation=='wrong-identity': value['records'][120]['identity']='another-state'
                else: value['cleanup_status']='FAIL'
                self.write(path,value)
                with self.assertRaises(ValueError): integrated.prepare_access(self.run,self.output,self.run)
                self.assertFalse(self.output.exists())

    def test_frozen_store_and_performance_binding_cannot_change(self):
        with contextlib.redirect_stdout(io.StringIO()): integrated.freeze_access(self.run)
        self.verify()
        performance=self.folder/'performance-result.json'; original=performance.read_bytes()
        performance.write_bytes(original+b'\n')
        with self.assertRaisesRegex(ValueError,'performance custody changed'): integrated.prepare_access(self.run,self.output,self.run)
        performance.write_bytes(original)
        frozen=self.folder/'frozen-measured-store/store.sqlite'
        with contextlib.closing(sqlite3.connect(frozen)) as db, db: db.execute("INSERT INTO content VALUES ('changed')")
        with self.assertRaisesRegex(ValueError,'frozen ordinary Store changed'): integrated.prepare_access(self.run,self.output,self.run)
        self.assertFalse(self.output.exists())

    def test_original_oracle_content_cannot_change(self):
        with contextlib.redirect_stdout(io.StringIO()): integrated.freeze_access(self.run)
        self.verify()
        Path(self.rows[0]['oracle']).write_text('{}')
        with self.assertRaisesRegex(ValueError,'original oracle changed'): integrated.prepare_access(self.run,self.output,self.run)
        self.assertFalse(self.output.exists())

if __name__=='__main__':unittest.main()
