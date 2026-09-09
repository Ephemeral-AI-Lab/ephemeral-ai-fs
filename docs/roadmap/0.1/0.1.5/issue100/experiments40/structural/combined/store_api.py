"""Actual-copy reader for the explicitly unsupported structural experiment."""
import collections
import importlib.util
from pathlib import Path
import sqlite3
import struct
import sys

ROOT = Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural')
sys.path.insert(0, str(ROOT/'metadata'))
import chain_api
spec = importlib.util.spec_from_file_location('structural_prior_reader', '/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/combined/store_api.py')
old = importlib.util.module_from_spec(spec)
spec.loader.exec_module(old)
check = old.check
d_api = old.d_api
MAGIC = b'LFSTRC\0\0'

class StoreReader(old.StoreReader):
    def __init__(self, path, cache_bytes=4*1024*1024, pack_cache_bytes=8*1024*1024):
        self.path = Path(path).resolve()
        self.db = sqlite3.connect(self.path.as_uri()+'?mode=ro&immutable=1', uri=True)
        check(self.db.execute('pragma user_version').fetchone()[0] == 9302, 'structural diagnostic version')
        self.loc = {i:(p,g,r,n) for i,n,p,g,r in self.db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects')}
        self.cache = collections.OrderedDict(); self.cache_bytes = 0; self.cache_limit = cache_bytes
        self.packs = collections.OrderedDict(); self.pack_bytes = 0; self.pack_limit = pack_cache_bytes
        self.z = old.fr.setup_decoder()

    def _physical(self, identity):
        check(identity in self.loc, 'missing object')
        p,g,r,n = self.loc[identity]
        data = self._pack(p)
        if data[:8] != MAGIC:
            if data[:8] == b'LFPACK\0\0' and int.from_bytes(data[8:12],'little') == 1:
                count = int.from_bytes(data[12:16],'little')
                check(1 <= count <= 256 and 0 <= g < count and len(data) >= 16+16*count, 'metadata group locator')
                decoded = struct.unpack_from('<I',data,16+16*g+8)[0]
                check(1 <= decoded <= 16384, 'metadata decoded group bound')
            return super()._physical(identity)
        version,count = struct.unpack_from('<II',data,8)
        check(version == 103 and 1 <= count <= 256 and 0 <= g < count and r == 0, 'structural locator')
        check(24 <= n < 131095 and len(data) >= 16+4*count, 'structural canonical/header bound')
        starts = list(struct.unpack_from('<'+'I'*count,data,16))+[len(data)]
        check(starts[0] == 16+4*count and all(a < b <= len(data) for a,b in zip(starts,starts[1:])), 'structural offsets')
        record = data[starts[g]:starts[g+1]]
        check(record and record[0] in (0,1), 'structural record kind')
        frame_length = len(record)-1-32*record[0]
        check(1 <= frame_length <= 135168, 'structural frame bound')
        return 4, record, count

    def read_canonical(self, identity):
        if isinstance(identity,str): identity = bytes.fromhex(identity)
        if identity in self.cache:
            self.cache.move_to_end(identity)
            return self.cache[identity]
        version,record,_ = self._physical(identity)
        if version not in (1,4): return super().read_canonical(identity)
        # Cold traversal always checks the full physical closure, even if a base is cached.
        nodes=[]; seen=set(); node=identity; raw_sum=encoded_sum=0
        while True:
            check(node not in seen, 'structural dependency cycle')
            seen.add(node)
            v,r,_ = self._physical(node)
            check(v == version and r and r[0] in (0,1), 'structural dependency role')
            n = self.loc[node][3]
            check(0 < n <= (8192 if version == 1 else 131094), 'structural object length')
            raw_sum += n; encoded_sum += len(r)
            check(raw_sum <= (131072 if version == 1 else 67108864), 'structural decoded closure')
            check(encoded_sum <= (17*8193 if version == 1 else 67108864), 'structural encoded closure')
            nodes.append((node,r,n))
            if r[0] == 0: break
            check(len(r) > (41 if version == 1 else 33), 'structural DELTA header')
            check(len(nodes) <= (16 if version == 1 else 50), 'structural edge limit')
            base = r[1:33]
            check(base in self.loc, 'structural missing base')
            if version == 1: check(self.loc[base][0] < self.loc[node][0], 'metadata chronological base')
            node = base
        previous = None; previous_id = None; canonical = None
        for node,r,n in reversed(nodes):
            if version == 1:
                check(len(r) <= 8193, 'metadata record bound')
                if r[0]:
                    check(chain_api.e.leaf(previous), 'metadata base leaf role')
                    canonical = chain_api.e.matcher.replay(r,previous_id,previous)
                    check(chain_api.e.leaf(canonical), 'metadata target leaf role')
                else: canonical = r[1:]
                previous = canonical
            else:
                frame = r[33:] if r[0] else r[1:]
                previous,workspace = old.fr.decode(self.z,frame,n-23,previous)
                canonical = b'LFSO\1'+struct.pack('>II',len(previous)+14,len(previous)+10)+b'LFS5SML\0\0\1'+previous
            self._put(node,canonical)
            previous_id = node
        return canonical
