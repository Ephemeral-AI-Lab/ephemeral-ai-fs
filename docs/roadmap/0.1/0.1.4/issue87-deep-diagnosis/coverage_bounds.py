#!/usr/bin/env python3
"""Derive checkpoint coverage bounds from sealed metadata, never a Store replay.

The set proof is explicit: retained objects must already be admitted. At every
checkpoint retained-prefix cardinality equals the acknowledged selected count.
Consequently the two sets coincide (append-only selected index for this run).
Every admitted eligible-shaped payload must pass candidate(), exclusions are zero,
and counted eligible equals newly admitted payload at every checkpoint. Thus no
extra eligible attempts fit those counts. Bounds identify bytes, not usefulness.
"""
import csv, hashlib, json, pathlib, sqlite3, sys

def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def read(p):return json.loads(p.read_text())

def bounds(sizes,n):
    assert 0 <= n <= len(sizes)
    sizes=sorted(sizes)
    return sum(sizes[:n]),sum(sizes[-n:]) if n else 0

def pack_checkpoints(c,out):
    rows=[]
    for pack,lo,hi,n,byte_count in c.execute('SELECT r.pack,min(l.first_retained_checkpoint),max(l.first_retained_checkpoint),count(*),p.bytes FROM records r JOIN logical l ON l.id=r.id JOIN packs p ON p.pack=r.pack WHERE r.selected=1 GROUP BY r.pack ORDER BY r.pack'):
        assert lo==hi and lo is not None
        rows.append({'pack_id':pack,'admission_checkpoint':lo,'selected_records_count':n,'pack_bytes':byte_count,'status':'derived','snapshot':'historical acknowledgement checkpoint','provenance':'retained-prefix equals selected-prefix at every acknowledgement; immutable selected locators; atomic pack insertion; all records selected','limitation':'No within-checkpoint sequence inferred; not derived from pack-ID order alone'})
    assert len(rows)==4915 and sum(r['selected_records_count'] for r in rows)==366141
    with (out/'derived-pack-checkpoints.csv').open('x',newline='') as f:
        w=csv.DictWriter(f,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
    return len(rows)

def main(evidence,out):
    out.mkdir(exist_ok=False)
    manifest=read(evidence/'manifest.sha256.json')
    inventory=evidence/'roles-inventory.sqlite'
    assert sha(inventory)==manifest['files'][inventory.name]['sha256']
    run=evidence.parent/'full157-m45-1'
    perf=read(run/'deepseek-full/performance-result.json')
    final=read(run/'verification-manifest.json')
    assert sha(run/'deepseek-full/performance-result.json')==final['deepseek-full/performance-result.json']
    c=sqlite3.connect(inventory.as_uri()+'?mode=ro&immutable=1',uri=True)
    graph={i:(n,b) for i,n,b in c.execute('SELECT first_retained_checkpoint,count(*),sum(bytes) FROM classified GROUP BY first_retained_checkpoint')}
    assert None not in graph
    rows=[]; prior_n=prior_b=0
    for i,s in enumerate([{'receipts':perf['ready']}]+perf['records']):
        a=next(r for r in s['receipts'] if r['kind']=='storage-smoke-allocation')
        stats={}
        for r in s['receipts']:
            if r['kind']=='storage-smoke-phase':
                for k,v in r.get('physical_storage',{}).items():stats[k]=stats.get(k,0)+v
        new_n=a['canonical_objects']-prior_n;new_b=a['canonical_bytes']-prior_b
        assert graph[i]==(new_n,new_b),(i,graph[i],new_n,new_b)
        prior_n=a['canonical_objects'];prior_b=a['canonical_bytes']
        # One checkpoint's scalar lengths only; no canonical payload retained.
        full=[];delta=[]
        for kind,size in c.execute('SELECT kind,bytes FROM classified WHERE role="payload_chunk" AND first_retained_checkpoint=?',(i,)):
            assert size+9<=65536
            (full if kind=='FULL' else delta).append(size)
        assert len(full)+len(delta)==stats['eligible_targets']
        assert len(delta)==stats['delta_selected']
        assert stats['memory_budget_skips']==0
        nohint=stats['targets_without_hints'];absent=stats['absent_predecessors']
        assert 0<=absent<=nohint<=len(full)
        hinted_full=len(full)-nohint
        row={'checkpoint':i,'new_objects_count':new_n,'new_canonical_bytes':new_b,'eligible_payload_count':len(full)+len(delta),'eligible_payload_canonical_bytes':sum(full)+sum(delta),'FULL_payload_count':len(full),'FULL_payload_canonical_bytes':sum(full),'DELTA_payload_count':len(delta),'DELTA_target_canonical_bytes':sum(delta),'nohint_count':nohint,'absent_predecessor_count':absent,'predecessor_present_nohint_count':nohint-absent,'hinted_FULL_count':hinted_full,'correspondence_reserved_bytes':stats['correspondence_reserved_bytes'],'correspondence_skip_events':stats['correspondence_budget_skips']}
        for name,n in [('nohint',nohint),('absent_predecessor',absent),('predecessor_present_nohint',nohint-absent),('hinted_FULL',hinted_full)]:
            row[name+'_canonical_bytes_lower'],row[name+'_canonical_bytes_upper']=bounds(full,n)
        rows.append(row)
    pack_checkpoints(c,out)
    totals={k:sum(r[k] for r in rows) for k in rows[0] if k!='checkpoint'}
    assert totals['eligible_payload_count']==86417 and totals['eligible_payload_canonical_bytes']==705162954
    assert totals['nohint_count']==77949 and totals['FULL_payload_canonical_bytes']==598565032
    with (out/'coverage-byte-bounds.csv').open('x',newline='') as f:
        w=csv.DictWriter(f,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
    summary={'status':'derived bounds from exact sealed metadata and receipts; source-flow assumptions independently reviewed','units':'*_bytes* integer canonical bytes; *_count integer unique objects; *_events precursor events; checkpoint ordinal','population':'unique newly admitted eligible payloads per Init/Commit checkpoint, demonstrated by retained-prefix/admission equality and complete candidate path','snapshot':'historical performance acknowledgement chronology derived against sealed post-verification graph','provenance':{'inventory_sha256':sha(inventory),'analysis_manifest_sha256':sha(evidence/'manifest.sha256.json'),'performance_sha256':sha(run/'deepseek-full/performance-result.json')},'proof':{'all158_retained_prefix_matches_acknowledged_growth':True,'all158_new_payload_counts_match_eligible_counts':True,'all158_new_DELTA_counts_match_selected_DELTA':True,'payloads_size_eligible':True,'optional_memory_skips':0,'set_reasoning':'L_i subset A_i; |L_i|=|A_i| at every i implies L_i=A_i. Thus first retained checkpoint equals selected admission checkpoint. Each new payload passes candidate at least once; eligible count equals new payload count so no extra attempts in those intervals. No intra-checkpoint order is inferred.','source_assumptions':['selected index append-only/no deletions in this run','all new selected payloads admitted through prepare_missing/candidate','no recorded optional memory exclusions; all payload sizes eligible','nohint and absent-predecessor targets become FULL; DELTA requires hints','absent_predecessor is a subset of nohint for this path']},'totals':totals,'bounds_definition':'At each checkpoint choose N smallest/largest newly selected FULL payload canonical lengths. Sum per-checkpoint extrema. Subcategories overlap; bounds are not jointly attainable by arbitrary summation.','limitations':['Bounds measure canonical bytes lacking hints, NOT compressed bytes or recoverable savings.','Prior existence does not imply useful correspondence/delta; no overlap and missing required span remain distinct unknowns.','No per-target historical outcome IDs exist; exact nohint byte total remains an interval.','No replay, candidate encoding, original Store access or original evidence mutation.']}
    (out/'coverage-byte-bounds.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps(totals,indent=2))

if __name__=='__main__':
    if sys.argv[1:] == ['--self-test']:
        assert bounds([9,1,4],0)==(0,0)
        assert bounds([9,1,4],2)==(5,13)
        assert bounds([9,1,4],3)==(14,14)
        try:bounds([1],2)
        except AssertionError:pass
        else:raise AssertionError('invalid subset cardinality accepted')
        print('coverage bounds self-test PASS')
    else:main(pathlib.Path(sys.argv[1]),pathlib.Path(sys.argv[2]))
