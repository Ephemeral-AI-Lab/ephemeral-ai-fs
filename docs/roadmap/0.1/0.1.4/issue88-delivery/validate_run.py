#!/usr/bin/env python3
"""D raw receipt + authenticated existing inventory validator. No payload decoder.

CLI: validate_run.py RUN SNAPSHOT INVENTORY EXPECTED NEW_OUTPUT
EXPECTED schema issue88-D-expected-v1:
  hashes: {performance,identity,snapshot,inventory,contract}: SHA256
  contract_path: committed contract path
  identity: nonempty subset of run identity.json source/host_identity fields
  inventory_authenticated: true (upstream assertion, never standalone proof)
  provenance: {path: inventory-proof JSON, sha256: its frozen hash}
    proof schema issue88-D-inventory-proof-v1: status PASS, snapshot_sha256,
    inventory_sha256, decoder_command nonempty argv, decoder_binary {path,sha256},
    custody {path,sha256}, quiescence {status:PASS,snapshot_mode:ro-immutable},
    all_selected_ids_authenticated:true, all_references_valid:true
  single_writer_append_only: true (upstream custody proof, never inferred from IDs)
  display_encoding: tagged33 (source default); digest32 only for an explicit suffix projection
  id_prefixes: {commit:18,layer:50} (source defaults)
Output exclusively created; INVALID preserves raw counters and null unique cohort.
"""
import argparse
import csv
import hashlib
import json
import pathlib
import re
import sqlite3
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / 'issue87-diagnostic-design'))
from validate_diagnostic import integer, pair, total, add, require

CURSOR = ('no_predecessor', 'missing_span', 'complete_empty', 'limited_empty', 'complete_hints', 'limited_hints')
TERMINALS = ('no_predecessor', 'missing_span', 'no_overlap', 'correspondence_limit', 'base', 'budget', 'no_delta', 'mixed_rejection', 'delta', 'unknown')
LIMITS = ('memory', 'file', 'operation', 'descriptor')
SIZE = (('lt64', 21, 63), ('lt256', 64, 255), ('lt1024', 256, 1023), ('lt4096', 1024, 4095), ('lt16384', 4096, 16383), ('lt65536', 16384, 65535), ('ge65536', 65536, None))
PAIR_NAMES = (('eligible', 'new_full', 'new_delta', 'race', 'nonfile_chunk') + CURSOR
    + tuple('occurrence_' + r for r in ('preexisting', 'missing', 'duplicate'))
    + tuple('terminal_' + r for r in TERMINALS)
    + tuple('size_' + r[0] for r in SIZE)
    + tuple('hints_' + str(i) for i in range(5))
    + tuple('limit_' + reason + '_' + age + suffix for reason in LIMITS for age in ('first', 'inherited') for suffix in ('', '_empty'))
    + tuple('event_' + r + '_budget' for r in ('fetch', 'match', 'instruction', 'memory')))
GAUGES = {'diag_selected_pack_last_id', 'diag_invalid'}
SCALARS = ('diag_invalid', 'diag_cursor_attached', 'diag_cursor_queries', 'diag_cursor_inherited',
    'diag_cursor_memory_limit', 'diag_cursor_file_limit', 'diag_cursor_operation_limit',
    'diag_cursor_descriptor_limit', 'diag_cursor_grants', 'diag_cursor_query_bytes',
    'diag_selected_pack_count', 'diag_selected_pack_last_id', 'diag_selected_pack_bytes',
    'diag_selected_pack_groups', 'diag_selected_pack_records', 'diag_selected_unlocated_records',
    'diag_occurrence_preexisting_grants', 'diag_occurrence_missing_grants', 'diag_occurrence_duplicate_grants',
    'diag_file_source_without_chunk')
EVENTS = ('base', 'budget', 'candidate', 'mixed_rejection')
FIELDS = tuple(dict.fromkeys(SCALARS + tuple('diag_' + n + suffix for n in PAIR_NAMES for suffix in ('_count', '_bytes'))
                           + tuple('diag_event_' + n + suffix for n in EVENTS for suffix in ('', '_bytes'))))


def sha(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def p(row, name):
    return pair({'count': row['diag_' + name + '_count'], 'canonical_bytes': row['diag_' + name + '_bytes']}, name)


def aggregate(receipts, strict=True):
    """Physical fields are phase deltas, except two endpoint gauges."""
    phases = [r for r in receipts if r.get('kind') == 'storage-smoke-phase']
    require(bool(phases), 'no phase receipts')
    names = [r['phase'] for r in phases]
    require(len(names) == len(set(names)), 'duplicate phase names; do not overwrite')
    result = {}
    for key in FIELDS:
        try:
            values = [integer(r['physical_storage'][key], key) for r in phases]
            result[key] = max(values) if key in GAUGES else sum(values)
        except (KeyError, ValueError, TypeError):
            if strict:
                raise
            result[key] = None
    return result


def validate_counters(row):
    errors = []
    def check(ok, message):
        if not ok:
            errors.append(message)
    try:
        for key in FIELDS:
            integer(row[key], key)
        for name in PAIR_NAMES:
            p(row, name)
        eligible = p(row, 'eligible')
        check(total(p(row, n) for n in CURSOR) == eligible, 'six cursor pairs do not partition eligible attempts')
        check(total(p(row, 'terminal_' + n) for n in TERMINALS) == add(p(row, 'new_full'), p(row, 'new_delta')), 'terminal winners do not conserve admitted file targets')
        check(add(total(p(row, 'terminal_' + n) for n in TERMINALS), p(row, 'race')) == eligible, 'terminal winners plus race do not conserve attempts')
        check(p(row, 'terminal_delta') == p(row, 'new_delta'), 'DELTA terminal differs from DELTA winner')
        check(p(row, 'occurrence_missing') == eligible, 'initial missing occurrence differs from eligible attempts')
        check(total(p(row, 'size_' + n) for n, _, _ in SIZE) == eligible, 'size marginals do not conserve attempts')
        for name, low, high in SIZE:
            n, b = p(row, 'size_' + name)
            check(b >= low * n and (high is None or b <= high * n), 'size bin byte bounds: ' + name)
        check(p(row, 'size_ge65536') == (0, 0), 'file chunk exceeds declared shape')
        check(eligible[1] <= 32789 * eligible[0], 'file chunk canonical bound')
        check(total(p(row, 'hints_' + str(i)) for i in range(5)) == eligible, 'hint marginals do not conserve')
        check(p(row, 'hints_0') == total(p(row, n) for n in CURSOR[:4]), 'zero-hint versus cursor partition')
        check(total(p(row, 'hints_' + str(i)) for i in range(1, 5)) == add(p(row, 'complete_hints'), p(row, 'limited_hints')), 'nonzero-hint versus cursor partition')
        limit_pairs = [p(row, 'limit_' + reason + '_' + age) for reason in LIMITS for age in ('first', 'inherited')]
        empty_pairs = [p(row, 'limit_' + reason + '_' + age + '_empty') for reason in LIMITS for age in ('first', 'inherited')]
        check(total(limit_pairs) == add(p(row, 'limited_empty'), p(row, 'limited_hints')), 'limit marginals do not conserve limited targets')
        check(total(empty_pairs) == p(row, 'limited_empty'), 'empty cause marginals do not conserve limited empty targets')
        for reason in LIMITS:
            for age in ('first', 'inherited'):
                whole = p(row, 'limit_' + reason + '_' + age)
                empty = p(row, 'limit_' + reason + '_' + age + '_empty')
                check(all(a <= b for a, b in zip(empty, whole)), 'empty cause not subset: ' + reason + '_' + age)
        grants = sum(row['diag_occurrence_' + r + '_grants'] for r in ('preexisting', 'missing', 'duplicate'))
        check(grants == row['diag_cursor_grants'], 'cursor and occurrence grant conservation')
        for route in ('preexisting', 'missing', 'duplicate'):
            check(row['diag_occurrence_' + route + '_grants'] <= 7 * p(row, 'occurrence_' + route)[0], 'per-occurrence grant ceiling')
        check(row['diag_cursor_grants'] <= 7 * row['diag_cursor_queries'], 'query grant bound')
        check(row['diag_cursor_inherited'] <= row['diag_cursor_queries'], 'inherited cursor events exceed queries')
        if p(row, 'race') == (0, 0):
            for terminal, cursor in zip(TERMINALS[:4], CURSOR[:4]):
                check(p(row, 'terminal_' + terminal) == p(row, cursor), 'unhinted terminal/cursor mismatch: ' + terminal)
        check(row['diag_invalid'] == 0, 'producer diagnostic invalidity reported')
        check(p(row, 'missing_span') == (0, 0), 'required file-payload span missing')
        check(p(row, 'terminal_unknown') == (0, 0), 'unknown terminal')
        check(row['diag_file_source_without_chunk'] == 0, 'file source tagged non-chunk')
        event_pairs = {}
        for name in EVENTS:
            event_pairs[name] = pair({'count': row['diag_event_' + name], 'canonical_bytes': row['diag_event_' + name + '_bytes']}, 'event_' + name)
            check(all(a <= b for a, b in zip(event_pairs[name], eligible)), 'target event exceeds eligible marginal: ' + name)
        check(all(a <= b for a, b in zip(event_pairs['mixed_rejection'], event_pairs['candidate'])), 'mixed-rejection event not within completed candidates')
        if p(row, 'race') == (0, 0):
            check(event_pairs['candidate'] == add(p(row, 'terminal_delta'), p(row, 'terminal_mixed_rejection')), 'completed candidate marginal differs from no-race delta/rejection terminals')
            check(event_pairs['mixed_rejection'] == p(row, 'terminal_mixed_rejection'), 'mixed-rejection event/terminal mismatch without races')
        for name in ('fetch', 'match', 'instruction', 'memory'):
            check(all(a <= b for a, b in zip(p(row, 'event_' + name + '_budget'), (row['diag_event_budget'], row['diag_event_budget_bytes']))), 'budget subevent exceeds budget event')
    except (ValueError, KeyError, TypeError) as exc:
        errors.append('missing/malformed raw field: ' + str(exc))
    return errors


def unique(rows, kind):
    found = [r for r in rows if r.get('kind') == kind]
    require(len(found) == 1, 'expected one ' + kind)
    return found[0]


def subset(actual, expected):
    return all(k in actual and (subset(actual[k], v) if isinstance(v, dict) else actual[k] == v) for k, v in expected.items())


def resolve_root(db, table, column, display, encoding):
    digits = 66 if encoding == 'tagged33' else 64
    require(encoding in ('tagged33', 'digest32') and re.fullmatch('[0-9a-fA-F]{'+str(digits)+'}', display) is not None, 'display ID encoding mismatch')
    operand = column if encoding == 'tagged33' else f'substr({column},2)'
    found = db.execute(f'SELECT {column},root_id FROM snap.{table} WHERE {operand}=?', (bytes.fromhex(display),)).fetchall()
    require(len(found) == 1 and len(found[0][0]) == 33, 'display ID mapping is not unique typed33 bytes')
    # Domain byte is checked by caller against frozen source ID prefix.
    return found[0]


class Graph:
    def __init__(self, db):
        self.db = db
        self.count = self.bytes = 0
        db.executescript('CREATE TABLE seen(id BLOB PRIMARY KEY,checkpoint INTEGER) WITHOUT ROWID; CREATE TABLE files(id BLOB PRIMARY KEY,checkpoint INTEGER) WITHOUT ROWID; CREATE TABLE frontier(id BLOB PRIMARY KEY) WITHOUT ROWID; CREATE TABLE file_frontier(id BLOB PRIMARY KEY) WITHOUT ROWID;')

    def retain(self, root, checkpoint):
        db = self.db
        db.execute('INSERT OR IGNORE INTO frontier VALUES(?)', (root,))
        while True:
            batch = db.execute('SELECT id FROM frontier LIMIT 512').fetchall()
            if not batch:
                break
            for (identifier,) in batch:
                db.execute('DELETE FROM frontier WHERE id=?', (identifier,))
                if not db.execute('INSERT OR IGNORE INTO seen VALUES(?,?)', (identifier, checkpoint)).rowcount:
                    continue
                obj = db.execute('SELECT bytes,role FROM objects WHERE id=?', (identifier,)).fetchone()
                require(obj is not None, 'retained graph child missing selected locator')
                self.count += 1
                self.bytes += obj[0]
                children = db.execute('SELECT child,use_label FROM inv.edges WHERE parent=?', (identifier,)).fetchall()
                db.executemany('INSERT OR IGNORE INTO frontier VALUES(?)', [(c,) for c, _ in children])
                if obj[1] == 'inode_record':
                    for child, label in children:
                        role = db.execute('SELECT role FROM objects WHERE id=?', (child,)).fetchone()
                        if label == 'content' and role == ('FileState',):
                            db.execute('INSERT OR IGNORE INTO file_frontier VALUES(?)', (child,))
        while True:
            batch = db.execute('SELECT id FROM file_frontier LIMIT 512').fetchall()
            if not batch:
                break
            for (identifier,) in batch:
                db.execute('DELETE FROM file_frontier WHERE id=?', (identifier,))
                if not db.execute('INSERT OR IGNORE INTO files VALUES(?,?)', (identifier, checkpoint)).rowcount:
                    continue
                obj = db.execute('SELECT role FROM objects WHERE id=?', (identifier,)).fetchone()
                require(obj is not None and obj[0] in ('FileState', 'extent_leaf', 'extent_branch', 'payload_chunk'), 'unexpected file content graph role')
                db.executemany('INSERT OR IGNORE INTO file_frontier VALUES(?)', db.execute('SELECT child FROM inv.edges WHERE parent=?', (identifier,)).fetchall())
        return self.count, self.bytes


def run(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    expected = json.loads(args.expected.read_text())
    require(expected['schema'] == 'issue88-D-expected-v1', 'expected schema')
    paths = {'performance': args.run / 'deepseek-full/performance-result.json', 'identity': args.run / 'identity.json', 'snapshot': args.snapshot, 'inventory': args.inventory, 'contract': pathlib.Path(expected['contract_path'])}
    actual_hashes = {key: sha(path) for key, path in paths.items()}
    require(actual_hashes == expected['hashes'], 'input or contract hash mismatch')
    require(expected['inventory_authenticated'] is True and expected['single_writer_append_only'] is True, 'upstream custody/decoder proof missing')
    proof_reference = expected['provenance']
    proof_path = pathlib.Path(proof_reference['path']).resolve()
    require(sha(proof_path) == proof_reference['sha256'], 'inventory proof artifact hash mismatch')
    proof = json.loads(proof_path.read_text())
    require(proof['schema'] == 'issue88-D-inventory-proof-v1' and proof['status'] == 'PASS', 'inventory proof incomplete')
    require(proof['snapshot_sha256'] == actual_hashes['snapshot'] and proof['inventory_sha256'] == actual_hashes['inventory'], 'proof does not bind this snapshot/inventory')
    require(isinstance(proof['decoder_command'], list) and bool(proof['decoder_command']) and all(isinstance(x, str) for x in proof['decoder_command']), 'actual decoder invocation missing')
    for key in ('decoder_binary', 'custody'):
        require(sha(pathlib.Path(proof[key]['path'])) == proof[key]['sha256'], key + ' proof hash mismatch')
    require(proof['quiescence']['status'] == 'PASS' and proof['quiescence']['snapshot_mode'] == 'ro-immutable', 'quiescence/read-only custody missing')
    require(proof['all_selected_ids_authenticated'] is True and proof['all_references_valid'] is True, 'upstream decoder validation incomplete')
    identity = json.loads(paths['identity'].read_text())
    require(bool(expected['identity']) and subset(identity, expected['identity']), 'frozen producer identity mismatch')
    data = json.loads(paths['performance'].read_text())
    require(data['status'] == data['cleanup_status'] == 'PASS', 'incomplete performance/cleanup')
    require(len(data['records']) == 157, 'full157 cardinality')
    db = sqlite3.connect(str(out / 'graph-analysis.sqlite'), uri=True)
    db.execute('ATTACH DATABASE ? AS inv', (args.inventory.resolve().as_uri() + '?mode=ro&immutable=1',))
    db.execute('ATTACH DATABASE ? AS snap', (args.snapshot.resolve().as_uri() + '?mode=ro&immutable=1',))
    db.executescript('PRAGMA cache_size=-16384; PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; CREATE TABLE objects(id BLOB PRIMARY KEY,bytes INTEGER,role TEXT,kind TEXT,pack INTEGER,grp INTEGER,rec INTEGER) WITHOUT ROWID; INSERT INTO objects SELECT id,bytes,role,kind,pack,grp,rec FROM inv.records WHERE selected=1; CREATE INDEX objects_pack ON objects(pack); CREATE TABLE packs(pack INTEGER PRIMARY KEY,bytes INTEGER,groups INTEGER); INSERT INTO packs SELECT pack,bytes,groups FROM inv.packs;')
    require(db.execute('SELECT count(*) FROM objects').fetchone()[0] == db.execute('SELECT count(*) FROM snap.objects').fetchone()[0], 'snapshot/inventory selected count mismatch')
    require(db.execute('SELECT count(*) FROM objects o LEFT JOIN snap.objects s ON s.object_id=o.id WHERE s.object_id IS NULL OR s.canonical_length!=o.bytes OR s.pack_id!=o.pack OR s.group_number!=o.grp OR s.record_number!=o.rec').fetchone()[0] == 0, 'snapshot/inventory selected locators mismatch')
    graph = Graph(db)
    ready = unique(data['ready'], 'storage-smoke-ready')
    typed, root = resolve_root(db, 'layers', 'layer_id', ready['layer_id'], expected.get('display_encoding', 'tagged33'))
    require(typed[0] == expected.get('id_prefixes', {'layer': 0x32})['layer'], 'layer domain prefix')
    checkpoints = [{'index': 0, 'receipts': data['ready'], 'root': root, 'mapping': ready['layer_id']}] + data['records']
    rows = []
    previous_last = 0
    previous_ack = (0, 0)
    all_errors = []
    for i, step in enumerate(checkpoints):
        require(step['index'] == i, 'checkpoint order')
        if i:
            typed, root = resolve_root(db, 'commits', 'commit_id', step['commit_id'], expected.get('display_encoding', 'tagged33'))
            require(typed[0] == expected.get('id_prefixes', {'commit': 0x12})['commit'], 'commit domain prefix')
        ack = unique(step['receipts'], 'storage-smoke-allocation')
        row = {'checkpoint': i, 'snapshot': 'acknowledgement before verifier', 'provenance': f'performance-result.json#{"ready" if i == 0 else "records/"+str(i-1)}'}
        row.update(source_sha=step.get('sha'), source_tree=step.get('tree'), source_oracle_sha256=step.get('oracle_sha256'), source_manifest_sha256=step.get('manifest_sha256'), previous_layerfs_mapping=ready['layer_id'] if i == 1 else checkpoints[i-1].get('commit_id') if i else None, resulting_layerfs_mapping=step.get('commit_id', ready['layer_id']), created=step.get('created'), source_identity_null_reason='Init has no source checkpoint or prior mapping' if i == 0 else None)
        errors = []
        try:
            row.update(aggregate(step['receipts'], strict=False))
            errors.extend(validate_counters(row))
        except (ValueError, KeyError, TypeError) as exc:
            errors.append('phase projection: ' + str(exc))
            row.update({k: None for k in FIELDS})
        row.update({key: value for key, value in ack.items() if key not in ('kind', 'label')})
        row.update({r['phase'] + '_elapsed_ns': r['elapsed_ns'] for r in step['receipts'] if r.get('kind') == 'storage-smoke-phase'})
        current = (ack['canonical_objects'], ack['canonical_bytes'])
        prefix = graph.retain(root, i)
        if prefix != current:
            errors.append('retained prefix count/bytes differ from acknowledgement')
        row.update(ack_objects=current[0], ack_canonical_bytes=current[1], retained_objects=prefix[0], retained_canonical_bytes=prefix[1], new_ack_objects=current[0]-previous_ack[0], new_ack_canonical_bytes=current[1]-previous_ack[1])
        last = row['diag_selected_pack_last_id']
        if last is not None:
            pack_count, pack_bytes, groups = db.execute('SELECT count(*),coalesce(sum(bytes),0),coalesce(sum(groups),0) FROM packs WHERE pack>? AND pack<=?', (previous_last, last)).fetchone()
            selected_count, selected_bytes = db.execute('SELECT count(*),coalesce(sum(bytes),0) FROM objects WHERE pack>? AND pack<=?', (previous_last, last)).fetchone()
            physical_records = db.execute('SELECT count(*) FROM inv.records WHERE pack>? AND pack<=?', (previous_last, last)).fetchone()[0]
            if last < previous_last or last-previous_last != row['diag_selected_pack_count'] or pack_count != row['diag_selected_pack_count']:
                errors.append('pack gauge/range/count mismatch or noncontiguous writer')
            for observed, wanted, message in ((pack_bytes, row['diag_selected_pack_bytes'], 'pack BLOB bytes'), (groups, row['diag_selected_pack_groups'], 'pack groups'), (physical_records, row['diag_selected_pack_records'], 'physical pack records'), (physical_records-selected_count, row['diag_selected_unlocated_records'], 'unlocated pack records')):
                if observed != wanted:
                    errors.append(message + ' mismatch')
            if (selected_count, selected_bytes) != (current[0]-previous_ack[0], current[1]-previous_ack[1]):
                errors.append('pack range selected count/bytes differ from acknowledgement growth')
            file_count, file_bytes, delta_count, delta_bytes = db.execute('SELECT count(*),coalesce(sum(o.bytes),0),coalesce(sum(o.kind="DELTA"),0),coalesce(sum(CASE WHEN o.kind="DELTA" THEN o.bytes ELSE 0 END),0) FROM objects o INDEXED BY objects_pack CROSS JOIN files f ON f.id=o.id WHERE o.role="payload_chunk" AND o.pack>? AND o.pack<=? AND f.checkpoint=?', (previous_last, last, i)).fetchone()
            row.update(pack_first_id=previous_last+1 if pack_count else None, pack_last_id=last, derived_selected_file_count=file_count, derived_selected_file_bytes=file_bytes, derived_selected_file_delta_count=delta_count, derived_selected_file_delta_bytes=delta_bytes)
            try:
                if (file_count, file_bytes) != p(row, 'eligible'):
                    errors.append('eligible attempts differ from newly selected file-graph payload cohort')
                if (delta_count, delta_bytes) != p(row, 'new_delta'):
                    errors.append('file-graph DELTA winners mismatch')
                if p(row, 'race') != (0, 0):
                    errors.append('race prevents unique cohort bijection')
            except (KeyError, ValueError, TypeError):
                errors.append('missing cohort counter prevents unique interpretation')
            previous_last = last
        row['status'] = 'INVALID' if errors else 'PASS'
        row['unique_eligible_count'] = None if errors else row['derived_selected_file_count']
        row['unique_eligible_bytes'] = None if errors else row['derived_selected_file_bytes']
        row['errors'] = errors
        rows.append(row)
        all_errors.extend(f'checkpoint {i}: {e}' for e in errors)
        previous_ack = current
        db.commit()
    if previous_last != db.execute('SELECT coalesce(max(pack),0) FROM packs').fetchone()[0]:
        all_errors.append('final pack gauge does not cover complete selected inventory')
    if sha(proof_path) != proof_reference['sha256']:
        all_errors.append('inventory proof changed during analysis')
    if not all(sha(path) == actual_hashes[key] for key, path in paths.items()):
        all_errors.append('input changed during analysis')
    with (out/'checkpoint-trajectory.csv').open('x', newline='') as f:
        keys = list(dict.fromkeys(k for row in rows for k in row))
        w = csv.DictWriter(f, fieldnames=keys); w.writeheader()
        w.writerows({k: json.dumps(v) if isinstance(v, list) else 'null' if v is None else v for k, v in row.items()} for row in rows)
    result = {'schema': 'issue88-D-raw-validation-v1', 'status': 'INVALID' if all_errors else 'PASS', 'input_hashes': actual_hashes, 'inventory_proof': proof_reference, 'decoder_command': proof['decoder_command'], 'proof_scope': 'verified bound provenance artifacts; canonical decoding was executed upstream, not repeated by this tool', 'upstream_identity_expected': expected['identity'], 'checkpoint_count': 158, 'errors': all_errors, 'unique_cohort': None if all_errors else {'objects_count': sum(r['unique_eligible_count'] for r in rows), 'canonical_bytes': sum(r['unique_eligible_bytes'] for r in rows)}, 'null_reason': 'Any custody/counter/graph/race failure prevents a unique target claim; raw attempt counters are preserved' if all_errors else None, 'field_contract': {'diag_selected_pack_last_id': 'absolute endpoint gauge; max across phase endpoints, never sum', 'diag_invalid': 'sticky endpoint; max', 'other_diag_fields': 'phase interval sums; count or canonical bytes named; grants count*131136 reserved bytes, not avoidable work', 'event_fields': 'overlapping target events; do not sum as exclusive outcomes', 'histograms': 'separate measured marginals; no invented joint cells', 'file_graph': 'inode_record content edge to FileState, then extent descendants; metadata value roots excluded', 'graph_snapshot': 'authenticated final immutable logical snapshot; incremental prefix union, not historical page reconstruction', 'pack_provenance': 'successful acknowledgement range gauges+counts+single-writer custody, joined to final immutable selected locators', 'units': 'integer count/bytes; identifiers and text explicitly named'}}
    with (out/'validation.json').open('x') as f:
        json.dump(result, f, indent=2); f.write('\n')
    db.close()
    print(json.dumps({'status': result['status'], 'errors': all_errors, 'unique_cohort': result['unique_cohort']}, indent=2))
    return 1 if all_errors else 0


def self_test():
    row = {key: 0 for key in FIELDS}
    for name in ('eligible', 'occurrence_missing', 'no_predecessor', 'terminal_no_predecessor', 'new_full', 'size_lt256', 'hints_0'):
        row['diag_' + name + '_count'] = 1
        row['diag_' + name + '_bytes'] = 64
    require(validate_counters(row) == [], 'valid synthetic marginal receipt')
    limited = dict(row)
    for name in ('no_predecessor', 'terminal_no_predecessor'):
        limited['diag_' + name + '_count'] = limited['diag_' + name + '_bytes'] = 0
    for name in ('limited_empty', 'terminal_correspondence_limit', 'limit_operation_first', 'limit_operation_first_empty'):
        limited['diag_' + name + '_count'] = 1
        limited['diag_' + name + '_bytes'] = 64
    limited['diag_cursor_queries'] = limited['diag_cursor_operation_limit'] = 1
    require(validate_counters(limited) == [], 'valid first-operation empty marginals')
    delta = dict(row)
    for name in ('no_predecessor', 'terminal_no_predecessor', 'new_full', 'hints_0'):
        delta['diag_' + name + '_count'] = delta['diag_' + name + '_bytes'] = 0
    for name in ('complete_hints', 'terminal_delta', 'new_delta', 'hints_1'):
        delta['diag_' + name + '_count'] = 1
        delta['diag_' + name + '_bytes'] = 64
    for name in ('base', 'candidate'):
        delta['diag_event_' + name] = 1
        delta['diag_event_' + name + '_bytes'] = 64
    require(validate_counters(delta) == [], 'valid hinted DELTA marginals')
    for key, value in [('diag_eligible_bytes', 65), ('diag_invalid', 1), ('diag_limit_operation_first_empty_count', 1), ('diag_cursor_grants', 1)]:
        bad = dict(row); bad[key] = value
        require(bool(validate_counters(bad)), 'bad synthetic case accepted: ' + key)
    phases = [{'kind': 'storage-smoke-phase', 'phase': name, 'physical_storage': dict(row, diag_selected_pack_last_id=7)} for name in ('exec', 'commit')]
    require(aggregate(phases)['diag_selected_pack_last_id'] == 7, 'gauge was summed')
    try:
        aggregate([phases[0], phases[0]])
    except ValueError:
        pass
    else:
        raise AssertionError('duplicate phase accepted')
    print('PASS: three valid marginal cases, four invalid receipts, endpoint gauge, duplicate phase rejection; no Store fixture')


if __name__ == '__main__':
    if sys.argv[1:] == ['--self-test']:
        self_test()
    else:
        parser = argparse.ArgumentParser(description=__doc__)
        for name in ('run', 'snapshot', 'inventory', 'expected', 'output'):
            parser.add_argument(name, type=pathlib.Path)
        args = parser.parse_args()
        new_output = not args.output.exists()
        try:
            code = run(args)
        except Exception as exc:
            if new_output and args.output.is_dir():
                with (args.output/'failure.json').open('x') as f:
                    json.dump({'schema': 'issue88-D-raw-validation-v1', 'status': 'INVALID', 'unique_cohort': None, 'null_reason': str(exc), 'performance_receipts': str(args.run/'deepseek-full/performance-result.json')}, f, indent=2)
                    f.write('\n')
            raise
        sys.exit(code)
