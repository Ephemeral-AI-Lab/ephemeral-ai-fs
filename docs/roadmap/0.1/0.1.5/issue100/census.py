"""Read-only pre-verification physical census; no history rebuild or Store mutation."""
import collections
import ctypes
import ctypes.util
import fcntl
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import struct
import sys
import time


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def small_closures(selected, schema_version):
    """Physical closure facts only; the public verifier owns authentication."""
    closures = {}
    bases = set()
    for identity, target in selected.items():
        if target[0] != 3:
            continue
        node, seen, decoded, encoded = identity, set(), 0, 0
        while True:
            assert node not in seen, 'SmallContent dependency cycle'
            seen.add(node)
            item = selected[node]
            assert item[0] == 3 and item[1] in (0, 1, 2)
            assert item[1] != 2 or schema_version >= 9, 'kind 2 requires schema 9'
            decoded += item[3] + 23
            encoded += item[4]
            if item[1] == 0:
                break
            base = selected[item[2]]
            assert base[0] == 3 and base[5] <= item[5], 'ineligible/future base'
            assert item[1] != 1 or base[1] == 0, 'legacy kind 1 requires FULL'
            bases.add(item[2])
            assert len(seen) <= 8, 'SmallContent depth exceeds 8 edges'
            node = item[2]
        if target[1] == 2:
            assert decoded <= 512 * 1024, 'decoded closure exceeds 512 KiB'
            assert encoded <= 256 * 1024, 'encoded closure exceeds 256 KiB'
        closures[identity] = (len(seen) - 1, decoded, encoded)
    return bases, closures


def self_check():
    # Census grammar checks, not an encoding or product benchmark.
    records = {0: (3, 0, None, 100, 109, 1),
               1: (3, 1, 0, 110, 60, 2),
               2: (3, 2, 1, 120, 61, 3)}
    bases, closures = small_closures(records, 9)
    assert bases == {0, 1} and closures[2] == (2, 399, 230)
    chain = {0: records[0], **{i: (3, 2, i-1, 100, 60, i+1) for i in range(1, 10)}}
    invalid = [({**records, 2: (3, 1, 1, 120, 61, 3)}, 9),
               (records, 8),
               ({**records, 1: (3, 2, 2, 110, 60, 3)}, 9),
               ({i: (*x[:3], 131071, *x[4:]) for i, x in chain.items() if i < 5}, 9),
               ({i: (*x[:4], 90000, x[5]) for i, x in records.items()}, 9),
               (chain, 9)]
    for rows, version in invalid:
        try:
            small_closures(rows, version)
        except AssertionError:
            continue
        raise AssertionError('invalid closure accepted')
    print('census closure self-check passed')


def main(run):
    start = time.monotonic_ns()
    case = json.loads((run / 'identity.json').read_text())['smoke']
    store = run / case / 'host-runtime/store.sqlite'
    manifest = json.loads((run / 'performance-manifest.json').read_text())
    digest = sha(store)
    assert digest == manifest[case + '/host-runtime/store.sqlite']
    db = sqlite3.connect(store.as_uri() + '?mode=ro&immutable=1', uri=True)
    db.execute('pragma cache_size=-8192')
    schema_version = db.execute('pragma user_version').fetchone()[0]
    libpath = ctypes.util.find_library('zstd')
    zstd = ctypes.CDLL(libpath)
    zstd.ZSTD_decompress.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t]
    zstd.ZSTD_decompress.restype = ctypes.c_size_t
    locators = {(p,g,r): (i,n) for i,n,p,g,r in db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects')}
    counts = collections.defaultdict(collections.Counter)
    selected = {}
    def record(version, p, g, ordinal, kind, raw, size, base, role):
        c = counts[role + ('_FULL' if kind == 0 else '_DELTA' if version != 2 else '_PREFIX')]
        c.update(objects=1, record_bytes=size, raw_bytes=raw)
        location = locators.get((p,g,ordinal))
        if location:
            identity, canonical = location
            selected[identity] = (version,kind,base,raw,size,p)
            c.update(selected=1, canonical_bytes=canonical)
        else:
            c.update(unlocated=1)
    for p, blob in db.execute('select pack_id,data from object_packs order by pack_id'):
        assert blob[:8] == b'LFPACK\0\0'
        version, groups = struct.unpack_from('<II',blob,8)
        assert version in (1,2,3) and 1 <= groups <= 256
        c = counts['pack_v'+str(version)]
        c.update(packs=1, bytes=len(blob), groups=groups, header_directory_bytes=16+16*groups)
        offset = 16+16*groups
        for g in range(groups):
            pos, encoded, decoded, codec = struct.unpack_from('<IIII',blob,16+16*g)
            assert pos == offset and codec in (0,1)
            offset += encoded
            body = blob[pos:offset]
            assert len(body) == encoded
            c.update(encoded_group_bytes=encoded, decoded_group_bytes=decoded)
            if version == 3:
                assert codec == 0 and encoded == decoded <= 196608
                kind,raw,frame = struct.unpack_from('<BII',body)
                assert kind in (0,1,2) and 0 < raw < 131072
                assert kind != 2 or schema_version >= 9
                header = 9+32*(kind != 0)
                assert len(body) == header+frame and 0 < frame <= 135168
                base = body[9:41] if kind else None
                record(version,p,g,0,kind,raw,len(body),base,'small')
                counts['small_DELTA' if kind else 'small_FULL'].update(frame_bytes=frame, record_header_bytes=header)
                counts['small_record_kinds'].update({str(kind): 1})
                assert locators[(p,g,0)][1] == raw+23
            else:
                if codec:
                    dest = ctypes.create_string_buffer(decoded)
                    assert zstd.ZSTD_decompress(dest,decoded,body,len(body)) == decoded
                    body = dest.raw
                assert len(body) == decoded
                n = struct.unpack_from('<I',body)[0]
                ends = struct.unpack_from('<'+'I'*n,body,4)
                begin = 4+4*n
                c.update(record_directory_bytes=begin)
                # Record ends are relative to the record area.
                for ordinal,relative_end in enumerate(ends):
                    end = 4+4*n+relative_end
                    r = body[begin:end]; assert r and end <= len(body)
                    kind = r[0]; assert kind in (0,1)
                    if version == 2:
                        raw = struct.unpack_from('<I',r,1)[0]; header = 5+32*kind
                        assert raw <= 32768 and len(r)>header
                        base = r[5:37] if kind else None
                        record(version,p,g,ordinal,kind,raw,len(r),base,'large_CDC')
                        counts['large_CDC_PREFIX' if kind else 'large_CDC_FULL'].update(frame_bytes=len(r)-header, record_header_bytes=header)
                    else:
                        raw = len(r)-1 if kind == 0 else struct.unpack_from('<I',r,33)[0]
                        base = r[1:33] if kind else None
                        record(version,p,g,ordinal,kind,raw,len(r),base,'metadata_legacy')
                        if kind == 0:
                            magic = r[14:22].rstrip(b'\0').decode('ascii','replace') if len(r)>22 else 'short'
                            counts['legacy_full_roles'].update({magic:1})
                    begin=end
                assert begin == len(body)
        assert offset == len(blob)
    assert len(selected) == len(locators)
    bases, closures = small_closures(selected, schema_version)
    for role, members in [('small_physical_bases', bases),
                          ('small_physical_FULL_bases', {b for b in bases if selected[b][1] == 0}),
                          ('small_physical_DELTA_bases', {b for b in bases if selected[b][1] != 0})]:
        # Each dependency object appears once; these are subsets of FULL/DELTA totals.
        counts[role].update(count=len(members), raw_bytes=sum(selected[b][3] for b in members),
                            record_bytes=sum(selected[b][4] for b in members))
    counts['small_depth_counts'].update(str(x[0]) for x in closures.values())
    for version, roles in [(1, ('metadata_legacy_FULL', 'metadata_legacy_DELTA')),
                           (2, ('large_CDC_FULL', 'large_CDC_PREFIX')),
                           (3, ('small_FULL', 'small_DELTA'))]:
        pack = counts['pack_v'+str(version)]
        assert pack['bytes'] == pack['header_directory_bytes'] + pack['encoded_group_bytes']
        assert pack['decoded_group_bytes'] == pack['record_directory_bytes'] + sum(counts[r]['record_bytes'] for r in roles)
    cursor = db.execute('select name,pagetype,count(*) pages,sum(pgsize) bytes,sum(payload) payload,sum(unused) unused from dbstat group by name,pagetype order by name,pagetype')
    pages = [dict(zip([x[0] for x in cursor.description],r)) for r in cursor]
    pragmas = {k:db.execute('pragma '+k).fetchone()[0] for k in ('user_version','page_size','page_count','freelist_count','auto_vacuum')}
    logical = pragmas['page_size']*pragmas['page_count']
    residual = logical-sum(p['bytes'] for p in pages)-pragmas['page_size']*pragmas['freelist_count']
    assert residual >= 0
    db.close()
    assert sha(store)==digest
    pack_bytes = sum(counts['pack_v'+str(v)]['bytes'] for v in (1,2,3))
    allocation = store.stat().st_blocks * 512
    reconciliation = dict(file_content_pack_bytes=counts['pack_v2']['bytes']+counts['pack_v3']['bytes'],
        metadata_legacy_pack_bytes=counts['pack_v1']['bytes'], all_pack_bytes=pack_bytes,
        sqlite_nonpack_bytes=logical-pack_bytes, sqlite_logical_bytes=logical,
        filesystem_allocation_difference_bytes=allocation-logical, store_allocated_bytes=allocation)
    assert pack_bytes <= logical
    result = dict(schema='issue100-census-v2',store=str(store),store_sha256=digest,store_inode=store.stat().st_ino,
        counts=dict(counts),pages=pages,pragmas=pragmas,sqlite_residual_bytes=residual,
        maximum_small_delta_depth=max((x[0] for x in closures.values()), default=0),
        maximum_small_decoded_canonical_closure_bytes=max((x[1] for x in closures.values()), default=0),
        maximum_small_retained_encoded_record_bytes=max((x[2] for x in closures.values()), default=0),
        reconciliation=reconciliation, elapsed_ns=time.monotonic_ns()-start,
        script_sha256=sha(Path(__file__)),decoder_library=libpath,decoder_library_sha256=sha(Path(libpath)),
        scope='Physical framing/locator/base accounting. Canonical authentication and exact visible bytes are qualified separately by same-Store historical verification.')
    with (run/'census.json').open('x') as stream: json.dump(result,stream,indent=2,sort_keys=True)
    print(json.dumps(result,sort_keys=True))

if __name__=='__main__':
    if sys.argv[1:] == ['--self-check']:
        self_check()
        raise SystemExit(0)
    with (Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        main(Path(sys.argv[1]).resolve())
