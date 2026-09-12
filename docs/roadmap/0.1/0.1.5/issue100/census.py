"""Read-only pre-verification physical census; no history rebuild or Store mutation."""
import collections
import argparse
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
        if target[0] not in (3, 4):
            continue
        node, seen, decoded, encoded = identity, set(), 0, 0
        while True:
            assert node not in seen, 'SmallContent dependency cycle'
            seen.add(node)
            item = selected[node]
            assert item[0] in (3, 4) and item[1] in (0, 1, 2)
            assert item[1] != 2 or schema_version >= 9, 'kind 2 requires schema 9'
            decoded += item[3] + 23
            encoded += item[4] + (8 if item[0] == 4 else 0)
            if item[1] == 0:
                break
            base = selected[item[2]]
            assert base[0] in (3, 4) and base[5] <= item[5], 'ineligible/future base'
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


def grouped_records(body):
    assert len(body) >= 4, 'group count missing'
    n = struct.unpack_from('<I', body)[0]
    assert 1 <= n <= 8191 and 4 + 4*n < len(body), 'record count/directory bound'
    ends = struct.unpack_from('<'+'I'*n, body, 4)
    begin = 4 + 4*n
    for ordinal, relative_end in enumerate(ends):
        end = 4 + 4*n + relative_end
        assert begin < end <= len(body), 'record directory range'
        yield ordinal, body[begin:end]
        begin = end
    assert begin == len(body), 'record directory endpoint'


def legacy_record(record):
    assert record and record[0] in (0, 1), 'legacy/metadata record kind'
    if record[0] == 0:
        assert 1 < len(record) <= 16*1024*1024+1, 'FULL record bound'
        return 0, len(record)-1, None
    assert 41 <= len(record) <= 65536, 'DELTA record bound'
    raw, count = struct.unpack_from('<II', record, 33)
    assert 1 <= raw <= 65536 and 1 <= count <= 8191, 'DELTA output/instruction bound'
    cursor = 41; produced = 0
    for _ in range(count):
        assert cursor < len(record), 'DELTA instruction missing'
        opcode = record[cursor]; cursor += 1
        assert opcode in (0, 1), 'DELTA instruction kind'
        cursor += 4 if opcode == 0 else 0
        assert cursor+4 <= len(record), 'DELTA instruction length missing'
        length = struct.unpack_from('<I', record, cursor)[0]; cursor += 4
        produced += length
        assert length > 0 and produced <= raw, 'DELTA instruction output'
        if opcode == 1: cursor += length
        assert cursor <= len(record), 'DELTA insert range'
    assert cursor == len(record) and produced == raw, 'DELTA program endpoint'
    return 1, raw, record[1:33]


def inspect_store(store):
    start = time.monotonic_ns()
    assert store.is_file() and not store.is_symlink(), 'regular closed Store required'
    assert not any(Path(str(store)+suffix).exists() for suffix in ('-wal','-shm','-journal')), 'closed Store has sidecars'
    digest = sha(store)
    db = sqlite3.connect(store.as_uri() + '?mode=ro&immutable=1', uri=True)
    try:
        return inspect_connection(db, store, digest, start)
    finally:
        db.close()
        assert sha(store) == digest, 'census changed Store bytes'


def inspect_connection(db, store, digest, start):
    db.execute('pragma cache_size=-8192')
    schema_version = db.execute('pragma user_version').fetchone()[0]
    libpath = zstd = None
    locators = {(p,g,r): (i,n) for i,n,p,g,r in db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects')}
    assert len(locators) == db.execute('select count(*) from objects').fetchone()[0], 'duplicate physical locator'
    pools = {}
    if schema_version >= 10:
        next_ordinal = 1
        for first,n,p,g,d in db.execute('select first_ordinal,count,pack_id,group_number,digest from metadata_value_groups order by first_ordinal'):
            assert first == next_ordinal and 1 <= n <= 165 and first+n <= 2**32 and len(d) == 32, 'metadata catalogue range'
            assert (p,g) not in pools, 'metadata catalogue group alias'
            pools[p,g] = n; next_ordinal += n
    counts = collections.defaultdict(collections.Counter)
    selected = {}
    observed_pools = set()
    largest_pack_read = largest_group_decode = 0
    def record(version, p, g, ordinal, kind, raw, size, base, role):
        c = counts[role + ('_FULL' if kind == 0 else '_DELTA' if version != 2 else '_PREFIX')]
        c.update(objects=1, record_bytes=size)
        if raw is not None: c.update(raw_bytes=raw)
        else: c.update(raw_length_unavailable=1)
        location = locators.get((p,g,ordinal))
        if location:
            identity, canonical = location
            selected[identity] = (version,kind,base,raw,size,p)
            c.update(selected=1, canonical_bytes=canonical)
        elif role == 'metadata_pool':
            c.update(catalogued_values=1)
        else:
            c.update(unlocated=1)
    for p, length in db.execute('select pack_id,length(data) from object_packs order by pack_id'):
        # Inspect the bounded header before materializing a potentially corrupt BLOB.
        assert 16 < length <= 16*1024*1024+41, 'pack BLOB bound'
        prefix = db.execute('select substr(data,1,16) from object_packs where pack_id=?', (p,)).fetchone()[0]
        version, groups = struct.unpack_from('<II',prefix,8)
        assert prefix[:8] == b'LFPACK\0\0' and version in range(1,7) and 1 <= groups <= 256
        assert version == 1 or length <= 256*1024, 'nonlegacy pack bound'
        assert version < 4 or schema_version >= 10, 'compact metadata/small framing requires schema10'
        directory = (4 if version == 4 else 16)*groups
        assert length > 16+directory, 'pack directory bound'
        blob = db.execute('select data from object_packs where pack_id=?', (p,)).fetchone()[0]
        largest_pack_read = max(largest_pack_read, len(blob))
        assert blob[:8] == b'LFPACK\0\0'
        c = counts['pack_v'+str(version)]
        c.update(packs=1, bytes=len(blob), groups=groups, header_directory_bytes=16+directory)
        offset = 16+directory
        for g in range(groups):
            if version == 4:
                pos = struct.unpack_from('<I',blob,16+4*g)[0]
                end = struct.unpack_from('<I',blob,20+4*g)[0] if g+1 < groups else len(blob)
                encoded = decoded = end-pos; codec = 0
                assert 2 <= encoded <= 33+135168, 'compact small group bound'
            else:
                pos, encoded, decoded, codec = struct.unpack_from('<IIII',blob,16+16*g)
            assert pos == offset and codec in (0,1) and encoded > 0 and decoded > 0
            assert (codec == 0 and encoded == decoded) or (codec == 1 and encoded <= decoded <= 65536)
            assert version not in (5,6) or decoded <= 16384, 'metadata group decoded bound'
            assert version != 2 or (codec == 0 and decoded <= 65536), 'native group bound'
            if decoded > 65536 or len(blob) > 256*1024:
                assert version in (1,3,4) and (version in (3,4) or (groups == 1 and codec == 0 and decoded <= 16*1024*1024+9))
            offset += encoded
            body = blob[pos:offset]
            assert len(body) == encoded
            c.update(encoded_group_bytes=encoded, decoded_group_bytes=decoded)
            largest_group_decode = max(largest_group_decode, decoded)
            if version in (3,4):
                assert codec == 0 and encoded == decoded <= 196608
                kind = body[0]
                location = locators.get((p,g,0))
                if version == 3:
                    assert len(body) >= 10, 'small record header'
                    kind,raw,frame = struct.unpack_from('<BII',body)
                    header = 9+32*(kind != 0)
                else:
                    raw = location[1]-23 if location else None
                    header = 1+32*(kind != 0); frame = len(body)-header
                assert kind in (0,1,2) and (raw is None or 0 < raw < 131072)
                assert kind != 2 or schema_version >= 9
                assert len(body) == header+frame and 0 < frame <= 135168
                base = body[header-32:header] if kind else None
                record(version,p,g,0,kind,raw,len(body),base,'small')
                counts['small_DELTA' if kind else 'small_FULL'].update(frame_bytes=frame, record_header_bytes=header)
                counts['small_record_kinds'].update({str(kind): 1})
                assert location is None or location[1] == raw+23
            else:
                if codec:
                    if zstd is None:
                        libpath = ctypes.util.find_library('zstd')
                        assert libpath, 'existing Zstandard decoder library unavailable'
                        zstd = ctypes.CDLL(libpath)
                        zstd.ZSTD_decompress.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t]
                        zstd.ZSTD_decompress.restype = ctypes.c_size_t
                    dest = ctypes.create_string_buffer(decoded)
                    assert zstd.ZSTD_decompress(dest,decoded,body,len(body)) == decoded
                    body = dest.raw
                assert len(body) == decoded
                n = struct.unpack_from('<I',body)[0]
                c.update(record_directory_bytes=4+4*n)
                pool = (p,g) in pools
                if pool:
                    assert version == 6 and n == pools[p,g], 'metadata pool group format/count'
                    assert not any((p,g,r) in locators for r in range(n)), 'metadata pool aliases canonical records'
                    observed_pools.add((p,g))
                role = 'metadata_pool' if pool else 'metadata_pooled' if version == 6 else 'metadata_compact' if version == 5 else 'metadata_legacy'
                if version in (1,5,6): counts[role+'_groups'].update(groups=1, encoded_bytes=encoded, decoded_bytes=decoded)
                for ordinal,r in grouped_records(body):
                    kind = r[0]; assert kind in (0,1)
                    if version == 2:
                        assert len(r) >= 6, 'native record header'
                        raw = struct.unpack_from('<I',r,1)[0]; header = 5+32*kind
                        assert raw <= 32768 and 0 < len(r)-header <= 33024
                        base = r[5:37] if kind else None
                        record(version,p,g,ordinal,kind,raw,len(r),base,'large_CDC')
                        counts['large_CDC_PREFIX' if kind else 'large_CDC_FULL'].update(frame_bytes=len(r)-header, record_header_bytes=header)
                    else:
                        kind,raw,base = legacy_record(r)
                        if pool:
                            assert kind == 0 and len(r) == 95 and r[1:6] == b'LFSO\x01'
                            assert r[6:14] == struct.pack('>II',85,81) and r[14:22] == b'LFSIVL1\0', 'metadata pool value grammar'
                        record(version,p,g,ordinal,kind,raw,len(r),base,role)
                        if kind == 0:
                            magic = r[14:22].rstrip(b'\0').decode('ascii','replace') if len(r)>22 else 'short'
                            counts['legacy_full_roles'].update({magic:1})
        assert offset == len(blob)
    assert len(selected) == len(locators)
    assert observed_pools == set(pools), 'unread metadata catalogue group'
    bases, closures = small_closures(selected, schema_version)
    for role, members in [('small_physical_bases', bases),
                          ('small_physical_FULL_bases', {b for b in bases if selected[b][1] == 0}),
                          ('small_physical_DELTA_bases', {b for b in bases if selected[b][1] != 0})]:
        # Each dependency object appears once; these are subsets of FULL/DELTA totals.
        counts[role].update(count=len(members), raw_bytes=sum(selected[b][3] for b in members),
                            record_bytes=sum(selected[b][4] for b in members))
    counts['small_depth_counts'].update(str(x[0]) for x in closures.values())
    native_bases = set()
    for item in selected.values():
        if item[0] == 2 and item[1] == 1:
            base = selected[item[2]]
            assert base[0] == 2 and base[1] == 0 and base[5] < item[5], 'native PREFIX requires an earlier physical FULL'
            native_bases.add(item[2])
    counts['large_CDC_physical_FULL_bases'].update(count=len(native_bases),
        raw_bytes=sum(selected[b][3] for b in native_bases), record_bytes=sum(selected[b][4] for b in native_bases))
    for version, roles in [(1, ('metadata_legacy_FULL', 'metadata_legacy_DELTA')),
                           (2, ('large_CDC_FULL', 'large_CDC_PREFIX')),
                           (5, ('metadata_compact_FULL', 'metadata_compact_DELTA')),
                           (6, ('metadata_pooled_FULL', 'metadata_pooled_DELTA', 'metadata_pool_FULL'))]:
        pack = counts['pack_v'+str(version)]
        assert pack['bytes'] == pack['header_directory_bytes'] + pack['encoded_group_bytes']
        assert pack['decoded_group_bytes'] == pack['record_directory_bytes'] + sum(counts[r]['record_bytes'] for r in roles)
    assert sum(counts['pack_v'+str(v)]['decoded_group_bytes'] for v in (3,4)) == sum(counts[r]['record_bytes'] for r in ('small_FULL','small_DELTA'))
    for version in (3,4):
        pack=counts['pack_v'+str(version)]
        assert pack['bytes'] == pack['header_directory_bytes'] + pack['encoded_group_bytes']
    cursor = db.execute('select name,pagetype,count(*) pages,sum(pgsize) bytes,sum(payload) payload,sum(unused) unused from dbstat group by name,pagetype order by name,pagetype')
    pages = [dict(zip([x[0] for x in cursor.description],r)) for r in cursor]
    pragmas = {k:db.execute('pragma '+k).fetchone()[0] for k in ('user_version','page_size','page_count','freelist_count','auto_vacuum')}
    logical = pragmas['page_size']*pragmas['page_count']
    residual = logical-sum(p['bytes'] for p in pages)-pragmas['page_size']*pragmas['freelist_count']
    assert residual >= 0
    pack_bytes = sum(counts['pack_v'+str(v)]['bytes'] for v in range(1,7))
    allocation = store.stat().st_blocks * 512
    apparent = store.stat().st_size
    assert apparent >= logical, 'database shorter than page count'
    reconciliation = dict(file_content_pack_bytes=sum(counts['pack_v'+str(v)]['bytes'] for v in (2,3,4)),
        metadata_legacy_pack_bytes=counts['pack_v1']['bytes'], metadata_compact_pack_bytes=counts['pack_v5']['bytes'],
        metadata_pooled_pack_bytes=counts['pack_v6']['bytes'], all_pack_bytes=pack_bytes,
        sqlite_nonpack_bytes=logical-pack_bytes, sqlite_logical_bytes=logical,
        database_trailing_bytes=apparent-logical, store_apparent_bytes=apparent,
        filesystem_allocation_difference_bytes=allocation-apparent, store_allocated_bytes=allocation)
    assert pack_bytes <= logical
    components = {
        'small_FULL_frame_bytes':counts['small_FULL']['frame_bytes'],
        'small_DELTA_frame_bytes':counts['small_DELTA']['frame_bytes'],
        'small_record_header_bytes':sum(counts[r]['record_header_bytes'] for r in ('small_FULL','small_DELTA')),
        'large_CDC_FULL_frame_bytes':counts['large_CDC_FULL']['frame_bytes'],
        'large_CDC_PREFIX_frame_bytes':counts['large_CDC_PREFIX']['frame_bytes'],
        'large_CDC_record_header_bytes':sum(counts[r]['record_header_bytes'] for r in ('large_CDC_FULL','large_CDC_PREFIX')),
        'large_CDC_record_directory_bytes':counts['pack_v2']['record_directory_bytes'],
        'pack_header_directory_bytes':sum(counts['pack_v'+str(v)]['header_directory_bytes'] for v in range(1,7)),
        **{role+'_encoded_group_bytes':counts[role+'_groups']['encoded_bytes'] for role in
           ('metadata_legacy','metadata_compact','metadata_pooled','metadata_pool')},
        'sqlite_pack_row_encoding_bytes':sum(p['payload'] for p in pages if p['name']=='object_packs')-pack_bytes,
        'sqlite_pack_page_overhead_bytes':sum(p['bytes']-p['payload']-p['unused'] for p in pages if p['name']=='object_packs'),
        'sqlite_pack_page_unused_bytes':sum(p['unused'] for p in pages if p['name']=='object_packs'),
        'sqlite_freelist_bytes':pragmas['freelist_count']*pragmas['page_size'],
        'sqlite_unattributed_page_bytes':residual,
        'database_trailing_bytes':apparent-logical,
        'filesystem_allocation_difference_bytes':allocation-apparent,
    }
    for page in pages:
        if page['name'] != 'object_packs':
            key='sqlite_tree:'+page['name']; components[key]=components.get(key,0)+page['bytes']
    assert all(value >= 0 for key,value in components.items() if key != 'filesystem_allocation_difference_bytes'), 'negative physical component'
    assert sum(components.values()) == allocation, 'complete allocated-byte conservation'
    result = dict(schema='issue100-census-v3',store=str(store),store_sha256=digest,store_inode=store.stat().st_ino,
        counts=dict(counts),pages=pages,pragmas=pragmas,sqlite_residual_bytes=residual,
        maximum_small_delta_depth=max((x[0] for x in closures.values()), default=0),
        maximum_small_decoded_canonical_closure_bytes=max((x[1] for x in closures.values()), default=0),
        maximum_small_retained_encoded_record_bytes=max((x[2] for x in closures.values()), default=0),
        reconciliation=reconciliation, allocated_components=components,
        read_bounds={'largest_pack_read_bytes':largest_pack_read,'largest_group_decoded_bytes':largest_group_decode,'sqlite_cache_bytes':8*1024*1024},
        elapsed_ns=time.monotonic_ns()-start,
        script_sha256=sha(Path(__file__)),decoder_library=libpath,decoder_library_sha256=sha(Path(libpath)) if libpath else None,
        omissions=['Group compression is shared: metadata FULL/DELTA record sizes are decoded sizes, not separately attributable compressed disk bytes.',
            'Unlocated compact-small records omit raw length; their stored frame/header bytes are counted, raw length is unavailable.',
            'Physical selected FULL/DELTA bases are subsets of retained records, never additive storage.',
            'Framing/base identity checks do not replace canonical authentication, DELTA reconstruction, or original-oracle verification.'],
        scope='Read-only physical framing/locator/base accounting. Canonical authentication and exact visible bytes are qualified separately by same-Store historical verification.')
    return result


def main(run, output=None):
    case = json.loads((run/'identity.json').read_text())['smoke']
    store = run/case/'host-runtime/store.sqlite'
    manifest = json.loads((run/'performance-manifest.json').read_text())
    performance = json.loads((run/case/'performance-result.json').read_text())
    assert performance['status'] == performance['cleanup_status'] == 'PASS', 'complete measured performance required'
    assert manifest[case+'/performance-result.json'] == sha(run/case/'performance-result.json'), 'performance result changed'
    if case == 'deepseek-full':
        assert [row['index'] for row in performance['records']] == list(range(1,158)), 'all157 measured states required'
    assert sha(store) == manifest[case+'/host-runtime/store.sqlite'], 'measured Store identity changed'
    result = inspect_store(store)
    result['performance_manifest_sha256'] = sha(run/'performance-manifest.json')
    result['performance_result_sha256'] = sha(run/case/'performance-result.json')
    with (output or run/'census.json').open('x') as stream: json.dump(result,stream,indent=2,sort_keys=True)
    print(json.dumps(result,sort_keys=True))

if __name__=='__main__':
    if sys.argv[1:] == ['--self-check']:
        self_check()
        raise SystemExit(0)
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('run',type=Path)
    parser.add_argument('--output',type=Path,help='new JSON output; default RUN/census.json')
    args=parser.parse_args()
    with (Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        main(args.run.resolve(),args.output)
