"""Small physical grammar/custody checks; no product, Git history or codec run."""
import contextlib
import importlib.util
import json
from pathlib import Path
import sqlite3
import struct
import tempfile
import unittest

import census

SPEC=importlib.util.spec_from_file_location('git157_verify',Path(__file__).parent/'experiments40/full157/git-baseline/verify.py')
GIT=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(GIT)


def canonical(value):
    return b'LFSO\x01'+struct.pack('>II',len(value)+4,len(value))+value


def group(records):
    ends=[]; end=0
    for record in records: end+=len(record); ends.append(end)
    return struct.pack('<I',len(records))+struct.pack('<'+'I'*len(ends),*ends)+b''.join(records)


def pack(version, groups):
    offset=16+(4 if version==4 else 16)*len(groups); directory=[]
    for body in groups:
        directory.append(struct.pack('<I',offset) if version==4 else struct.pack('<IIII',offset,len(body),len(body),0))
        offset+=len(body)
    return b'LFPACK\0\0'+struct.pack('<II',version,len(groups))+b''.join(directory)+b''.join(groups)


class PhysicalAttributionTests(unittest.TestCase):
    def setUp(self):
        self.directory=tempfile.TemporaryDirectory(); self.addCleanup(self.directory.cleanup)
        self.store=Path(self.directory.name)/'store.sqlite'
        with contextlib.closing(sqlite3.connect(self.store)) as db, db:
            db.executescript('''PRAGMA user_version=10;
                CREATE TABLE object_packs(pack_id INTEGER PRIMARY KEY,data BLOB);
                CREATE TABLE objects(object_id BLOB PRIMARY KEY,canonical_length INTEGER,pack_id INTEGER,group_number INTEGER,record_number INTEGER,UNIQUE(pack_id,group_number,record_number));
                CREATE TABLE metadata_value_groups(first_ordinal INTEGER PRIMARY KEY,count INTEGER,pack_id INTEGER,group_number INTEGER,digest BLOB);''')

    def add(self, number, blob, locators=(), pool=None):
        with contextlib.closing(sqlite3.connect(self.store)) as db, db:
            db.execute('INSERT INTO object_packs VALUES (?,?)',(number,blob))
            for identity, length, group_number, record_number in locators:
                db.execute('INSERT INTO objects VALUES (?,?,?,?,?)',(identity,length,number,group_number,record_number))
            if pool is not None:
                db.execute('INSERT INTO metadata_value_groups VALUES (1,?,?,?,?)',(pool[1],number,pool[0],b'x'*32))

    def populated(self):
        self.add(1,pack(3,[b'\0'+struct.pack('<II',8,3)+b'old']),[(b'A'*32,31,0,0)])
        self.add(2,pack(4,[b'\2'+b'A'*32+b'delta']),[(b'B'*32,32,0,0)])
        self.add(3,pack(4,[b'\0orphan',b'\0new']),[(b'C'*32,33,1,0)])
        self.add(4,pack(2,[group([b'\0'+struct.pack('<I',10)+b'native'])]),[(b'D'*32,33,0,0)])
        self.add(5,pack(2,[group([b'\1'+struct.pack('<I',10)+b'D'*32+b'prefix'])]),[(b'E'*32,33,0,0)])
        value=b'\1'+(1).to_bytes(8,'big')+b'C'*32+b'M'*32
        header=b'LFS6INT\0'+struct.pack('>HBBB HQQ',1,7,0,0,1,1,81)
        leaf=canonical(header+(1).to_bytes(8,'big')+value)
        self.add(6,pack(5,[group([b'\0'+leaf])]),[(b'F'*32,len(leaf),0,0)])
        pooled=leaf[:44]+(1).to_bytes(8,'big')+(1).to_bytes(4,'big')
        values=group([b'\0'+canonical(b'LFSIVL1\0'+value)])
        self.add(7,pack(6,[group([b'\0'+pooled]),values]),[(b'G'*32,len(leaf),0,0)],pool=(1,1))
        self.add(8,pack(1,[group([b'\0'+canonical(b'legacy')])]),[(b'H'*32,19,0,0)])

    def test_all_six_versions_reconcile_and_bases_are_nonadditive(self):
        self.populated(); before=census.sha(self.store)
        result=census.inspect_store(self.store)
        self.assertEqual(census.sha(self.store),before)
        self.assertEqual(sum(result['allocated_components'].values()),self.store.stat().st_blocks*512)
        self.assertEqual(result['reconciliation']['store_apparent_bytes'],self.store.stat().st_size)
        self.assertEqual(result['counts']['small_FULL']['raw_length_unavailable'],1)
        self.assertEqual(result['counts']['small_physical_FULL_bases']['count'],1)
        self.assertEqual(result['counts']['large_CDC_physical_FULL_bases']['count'],1)
        self.assertEqual(result['counts']['metadata_pool_groups']['groups'],1)
        self.assertEqual(result['counts']['metadata_pooled_groups']['groups'],1)
        self.assertEqual(result['counts']['metadata_pool_FULL']['catalogued_values'],1)
        self.assertGreater(result['allocated_components']['metadata_pool_encoded_group_bytes'],0)
        self.assertEqual(result['maximum_small_retained_encoded_record_bytes'],58)

    def test_compact_unlocated_frame_needs_no_guessed_raw_length(self):
        self.add(1,pack(4,[b'\0unlocated']))
        result=census.inspect_store(self.store)
        self.assertEqual(result['counts']['small_FULL']['frame_bytes'],9)
        self.assertEqual(result['counts']['small_FULL']['raw_length_unavailable'],1)
        self.assertEqual(result['maximum_small_delta_depth'],0)

    def test_bad_directories_counts_and_sidecars_fail_without_mutation(self):
        self.populated()
        with contextlib.closing(sqlite3.connect(self.store)) as db:
            original=db.execute('SELECT data FROM object_packs WHERE pack_id=7').fetchone()[0]
        malformed=bytearray(original); struct.pack_into('<I',malformed,16,0)
        with contextlib.closing(sqlite3.connect(self.store)) as db, db:
            db.execute('UPDATE object_packs SET data=? WHERE pack_id=7',(malformed,))
        before=census.sha(self.store)
        with self.assertRaises(AssertionError): census.inspect_store(self.store)
        self.assertEqual(census.sha(self.store),before)
        with contextlib.closing(sqlite3.connect(self.store)) as db, db:
            db.execute('UPDATE object_packs SET data=? WHERE pack_id=7',(original,))
            db.execute('UPDATE metadata_value_groups SET count=2')
        with self.assertRaisesRegex(AssertionError,'count'): census.inspect_store(self.store)
        Path(str(self.store)+'-journal').write_bytes(b'not-closed')
        with self.assertRaisesRegex(AssertionError,'sidecars'): census.inspect_store(self.store)

    def test_bounded_header_rejects_large_nonlegacy_blob_before_full_read(self):
        blob=pack(4,[b'\0'+b'x'*(256*1024)])
        self.add(1,blob)
        with self.assertRaisesRegex(AssertionError,'nonlegacy pack bound'): census.inspect_store(self.store)

    def test_record_directory_and_delta_program_require_complete_consumption(self):
        self.assertEqual(list(census.grouped_records(group([b'\0one',b'\0two']))),[(0,b'\0one'),(1,b'\0two')])
        delta=b'\1'+b'A'*32+struct.pack('<II',3,1)+b'\1'+struct.pack('<I',3)+b'abc'
        self.assertEqual(census.legacy_record(delta),(1,3,b'A'*32))
        for value in (delta[:-1],delta+b'x',delta[:37]+struct.pack('<I',8192)+delta[41:]):
            with self.assertRaises(AssertionError): census.legacy_record(value)
        with self.assertRaises(AssertionError): list(census.grouped_records(group([b'\0one'])+b'trailing'))


class GitCustodyTests(unittest.TestCase):
    def test_all_original_checkpoints_and_oracle_hashes_are_required(self):
        with tempfile.TemporaryDirectory() as directory:
            oracle=Path(directory)/'oracle.json'; oracle.write_text('{}')
            originals=[{'index':i,'sha':str(i),'tree':'tree'+str(i)} for i in range(1,158)]
            mapped=[{'index':r['index'],'source_sha':r['sha'],'tree':r['tree'],'git_commit':'commit'+str(r['index'])} for r in originals]
            fixture=[{**r,'full157_index':r['index'],'oracle':str(oracle),'oracle_sha256':GIT.sha(oracle)} for r in originals]
            self.assertEqual(len(GIT.mappings(originals,mapped,fixture)),157)
            for broken in (mapped[:-1],[*mapped[:120],{**mapped[120],'tree':'wrong'},*mapped[121:]]):
                with self.assertRaises(AssertionError): GIT.mappings(originals,broken,fixture)
            oracle.write_text('{"changed":true}')
            with self.assertRaisesRegex(AssertionError,'oracle changed'): GIT.mappings(originals,mapped,fixture)

    def test_inventory_binds_exact_bytes_and_rejects_symlinks(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory); (root/'config').write_bytes(b'original')
            before=GIT.inventory(root)
            self.assertEqual(before['config']['bytes'],8)
            self.assertEqual(GIT.inventory(root),before)
            (root/'alias').symlink_to(root/'config')
            with self.assertRaisesRegex(AssertionError,'symlink'): GIT.inventory(root)


if __name__ == '__main__': unittest.main()
