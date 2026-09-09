"""Read every candidate namespace and compare original157 path/content oracles."""
import argparse, hashlib, json, struct, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
FIXTURE = Path('/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data')
SOURCE = Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full')
sys.path.insert(0, str(ROOT/'combined'))
import store_api as physical

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def value(canonical):
    assert canonical[:5] == b'LFSO\1'
    assert struct.unpack_from('>II', canonical, 5) == (len(canonical)-9, len(canonical)-13)
    return canonical[13:]

class Inventory:
    def __init__(self, reader): self.reader = reader
    def __getitem__(self, identity): return self.reader.read_canonical(identity)

class Files:
    def __init__(self, reader):
        self.reader = reader
        self.summaries = {}
        self.modes = {}
        self.unique_logical_bytes = 0

    def extents(self, identity, depth=0):
        assert depth <= 31
        v = value(self.reader.read_canonical(identity))
        assert v[:10] == b'LFS4MAP\0\0\3' and v[12] == 0
        kind, level = v[10:12]
        count = int.from_bytes(v[13:15], 'big')
        logical, extent_count = struct.unpack_from('>QQ', v, 15)
        seen_bytes = seen_extents = 0
        if kind == 8:
            assert level == 0 and len(v) == 31 + 40*count and count == extent_count
            for p in range(31, len(v), 40):
                chunk = value(self.reader.read_canonical(v[p:p+32]))
                assert chunk[:8] == b'LFS4CHK\0'
                start, size = struct.unpack_from('>II', v, p+32)
                raw = chunk[8:]
                assert size > 0 and start+size <= len(raw) <= 32768
                seen_bytes += size
                seen_extents += 1
                yield raw[start:start+size]
        else:
            assert kind == 9 and level > 0 and len(v) == 31+48*count
            previous_bytes = previous_extents = 0
            for p in range(31, len(v), 48):
                end, end_extents = struct.unpack_from('>QQ', v, p)
                child = v[p+16:p+48]
                child_value = value(self.reader.read_canonical(child))
                assert child_value[11]+1 == level
                assert int.from_bytes(child_value[23:31], 'big') == end_extents-previous_extents
                size = 0
                for part in self.extents(child, depth+1):
                    size += len(part)
                    yield part
                assert size == end-previous_bytes
                previous_bytes, previous_extents = end, end_extents
            seen_bytes, seen_extents = previous_bytes, previous_extents
        assert seen_bytes == logical and seen_extents == extent_count

    def blocks(self, identity):
        v = value(self.reader.read_canonical(identity))
        if v[:8] == b'LFS5SML\0':
            assert v[8:10] == b'\0\1' and 0 < len(v)-10 < 131072
            yield v[10:]
            return
        assert len(v) == 93 and v[:12] == b'LFS4MAP\0\0\3\x0a\0'
        length, count = struct.unpack_from('>QQ', v, 12)
        tree = v[61:93]
        tv = value(self.reader.read_canonical(tree))
        assert tv[11] == v[28] and int.from_bytes(tv[23:31], 'big') == count
        size = 0
        for part in self.extents(tree):
            size += len(part)
            yield part
        assert size == length

    def info(self, identity):
        if identity not in self.summaries:
            size = 0
            h = hashlib.sha256()
            for part in self.blocks(identity):
                size += len(part)
                h.update(part)
            self.summaries[identity] = (size, h.hexdigest())
            self.unique_logical_bytes += size
        return self.summaries[identity]

    def metadata_entries(self, identity, depth=0):
        assert depth <= 31
        v = value(self.reader.read_canonical(identity))
        assert v[:8] == b'LFS4MET\0'
        kind, level, flags = v[10:13]
        count = int.from_bytes(v[13:15], 'big')
        assert flags == 0 and ((kind == 9 and level == 0) or (kind == 10 and level > 0))
        p = 31
        entries = []
        for _ in range(count):
            width = int.from_bytes(v[p:p+2], 'big'); p += 2
            domain = v[p:p+width]; p += width
            width = int.from_bytes(v[p:p+2], 'big'); p += 2
            key = v[p:p+width]; p += width
            if kind == 9:
                assert v[p] == 1; p += 1
            reference = v[p:p+32]; p += 32
            assert len(reference) == 32
            if kind == 9: entries.append(((domain, key), reference))
            else: entries.extend(self.metadata_entries(reference, depth+1))
        assert p == len(v) and len(entries) == int.from_bytes(v[15:23], 'big')
        assert len(dict(entries)) == len(entries)
        return entries

    def mode(self, identity, kind):
        if (identity, kind) not in self.modes:
            root = dict(self.metadata_entries(identity))[(b'portable', b'mode')]
            raw = b''.join(self.blocks(root))
            assert len(raw) == 4
            permission = int.from_bytes(raw, 'big')
            assert permission & ~0o7777 == 0
            # Original Git-derived oracles use 120000; still check actual link permissions.
            if kind == 3: assert permission == 0o777
            self.modes[identity, kind] = '120000' if kind == 3 else format({1:0o100000,2:0o40000}[kind] | permission, 'o')
        return self.modes[identity, kind]

def observe(reader, files, root):
    inv = Inventory(reader)
    namespace = value(inv[root])
    assert namespace[:8] == b'LFS6FSR\0' and len(namespace) == 116
    inode_root, table_root = namespace[76:84], namespace[84:116]
    pairs = physical.d_api.decode_table(inv, table_root)
    inodes = dict(pairs)
    assert len(inodes) == len(pairs) and inode_root in inodes
    observed = {}
    seen_directories = set()
    references = {}
    pending = [(b'', inode_root)]
    while pending:
        path, inode = pending.pop()
        iv = inodes[inode]
        assert len(iv) == 73
        kind = iv[0]
        content, metadata = iv[9:41], iv[41:73]
        if path: references[inode] = references.get(inode,0)+1
        mode = files.mode(metadata, kind)
        if kind == 2:
            assert inode not in seen_directories, 'directory cycle/hardlink'
            seen_directories.add(inode)
            for name, child in physical.d_api.decode_directory(inv, content):
                assert name not in (b'',b'.',b'..') and b'/' not in name and b'\0' not in name
                pending.append(((path+b'/' if path else b'')+name, child))
            row = [mode,0,'-']
        elif kind == 1:
            length, digest = files.info(content)
            row = [mode,length,digest]
        else:
            assert kind == 3
            v = value(inv[content])
            assert v[:12] == b'LFS4LNK\0\0\1\5\0'
            length = int.from_bytes(v[12:14], 'big')
            assert len(v) == 14+length
            row = [mode,length,hashlib.sha256(v[14:]).hexdigest()]
        if path:
            key = path.hex()
            assert key not in observed
            observed[key] = row
    assert len(references)+1 == len(inodes)
    for inode, iv in pairs:
        assert int.from_bytes(iv[1:9], 'big') == references.get(inode,0)
    return observed

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--limit', type=int, default=157)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    assert 1 <= args.limit <= 157 and not args.output.exists()
    summary = json.loads((ROOT/'combined/result.json').read_text())
    result = summary['results']['D-B-CDC']
    path = Path(result['path'])
    before = sha(path)
    assert before == result['sha256']
    identity = json.loads((ROOT/'combined/identity-map.json').read_text())
    checkpoints = json.loads((FIXTURE/'checkpoint-manifest.json').read_text())['checkpoints']
    original = json.loads((SOURCE/'verification-result.json').read_text())['records']
    assert len(checkpoints) == len(original) == 157
    reader = physical.StoreReader(path, cache_bytes=4*1024*1024, pack_cache_bytes=8*1024*1024)
    files = Files(reader)
    rows = []
    started = time.monotonic()
    try:
        for checkpoint, receipt in list(zip(checkpoints,original))[:args.limit]:
            step = checkpoint['index']
            assert receipt['index'] == step
            new_commit = bytes.fromhex(identity['commits'][receipt['identity']])
            row = reader.db.execute('select root_id from commits where commit_id=?',[new_commit]).fetchone()
            assert row is not None
            oracle_path = FIXTURE/'oracles'/(checkpoint['sha']+'.json')
            expected = json.loads(oracle_path.read_text())
            observed = observe(reader,files,row[0])
            if expected != observed:
                differing = sorted(k for k in set(expected)|set(observed) if expected.get(k) != observed.get(k))
                raise AssertionError((step,[(k,expected.get(k),observed.get(k)) for k in differing[:5]]))
            logical = sum(v[1] for v in observed.values())
            assert logical == checkpoint['logical_bytes']
            rows.append(dict(index=step,source_sha=checkpoint['sha'],commit_id=new_commit.hex(),paths=len(observed),logical_bytes=logical,oracle_sha256=sha(oracle_path),status='PASS'))
            if step == 1 or step % 10 == 0 or step == args.limit:
                print(f'PASS {step}/{args.limit}: {len(observed)} paths, {logical} logical bytes',flush=True)
    finally:
        reader.close()
    after = sha(path)
    assert before == after
    paths = sum(r['paths'] for r in rows)
    logical = sum(r['logical_bytes'] for r in rows)
    if args.limit == 157: assert paths == 904143 and logical == 4936693030
    output = dict(scope='Standalone exact original-oracle verification of assembled offline candidate; bounded caches and authenticated-root digest reuse; not public API checkout/performance',status='PASS',states=len(rows),path_states=paths,logical_bytes=logical,unique_file_roots=len(files.summaries),unique_file_bytes_hashed=files.unique_logical_bytes,copy_sha256_before=before,copy_sha256_after=after,script_sha256=sha(__file__),reader_sha256=sha(ROOT/'combined/store_api.py'),manifest_sha256=sha(FIXTURE/'checkpoint-manifest.json'),elapsed_seconds=time.monotonic()-started,records=rows)
    args.output.write_text(json.dumps(output,indent=2)+'\n')
    print(json.dumps({k:v for k,v in output.items() if k!='records'},indent=2))

if __name__ == '__main__': main()
