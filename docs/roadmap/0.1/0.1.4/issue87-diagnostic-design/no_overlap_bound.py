#!/usr/bin/env python3
"""Oracle-only upper bound on completed legitimate no-overlap; no payload/Store reads."""
import csv, hashlib, json, pathlib, sys

def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def regular(v):return v is not None and v[0] in ('100644','100755')

def growth(previous,current):
    sizes=[max(0,v[1]-previous[k][1]) for k,v in current.items() if regular(v) and regular(previous.get(k))]
    return sum(sizes),sum(n>0 for n in sizes)

def main(run,prior_bounds,out):
    out.mkdir(exist_ok=False)
    performance=json.loads((run/'deepseek-full/performance-result.json').read_text())
    prior=json.loads((prior_bounds/'manifest.sha256.json').read_text())
    for name in ['coverage-byte-bounds.csv','coverage-byte-bounds.json']:
        assert sha(prior_bounds/name)==prior['files'][name]['sha256']
    bounds={int(r['checkpoint']):r for r in csv.DictReader((prior_bounds/'coverage-byte-bounds.csv').open())}
    previous={};rows=[];oracles={}
    for s in performance['records']:
        oracle=pathlib.Path(s['oracle']);digest=sha(oracle)
        assert digest==s['oracle_sha256']
        oracles[str(s['index'])]=digest
        current=json.loads(oracle.read_text())
        tail,files=growth(previous,current);i=s['index'];b=bounds[i]
        n=int(b['predecessor_present_nohint_count']);lo=int(b['predecessor_present_nohint_canonical_bytes_lower']);hi=int(b['predecessor_present_nohint_canonical_bytes_upper'])
        bound=min(hi,tail+21*n)
        rows.append({'checkpoint':i,'samepath_regular_tail_growth_payload_bytes':tail,'growing_file_occurrences_count':files,'predecessor_present_nohint_count':n,'predecessor_present_nohint_lower_canonical_bytes':lo,'predecessor_present_nohint_upper_canonical_bytes':hi,'completed_nooverlap_upper_canonical_bytes':bound,'not_completed_nooverlap_lower_canonical_bytes':max(0,lo-bound)})
        previous=current
    totals={k:sum(r[k] for r in rows) for k in rows[0] if k!='checkpoint'}
    with (out/'no-overlap-bounds.csv').open('x',newline='') as f:
        w=csv.DictWriter(f,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
    summary={'status':'source-supported derived upper/lower bounds; explicit valid-span and samepath-predecessor premises','units':'integer payload/canonical bytes as named; counts are file occurrences or unique targets as named','snapshot':'original157 performance input/oracle states; no Store census','population':'predecessor-present nohint initially-missing eligible payload cohort established by prior prefix proof','provenance':{'original_performance_sha256':sha(run/'deepseek-full/performance-result.json'),'prior_bounds_manifest_sha256':sha(prior_bounds/'manifest.sha256.json'),'oracle_sha256_by_checkpoint':oracles},'totals':totals,'equations':['G_i = sum over samepath regular-file pairs max(new_length-old_length,0)','completed nooverlap target with valid whole-payload span has start >= prior EOF, hence its payload lies in G_i','C_i <= min(prior nohint-with-predecessor upper bytes, G_i + 21 * predecessor-present-nohint target count_i)','not-completed-nooverlap_i >= max(0, prior nohint-with-predecessor lower bytes - C_i)'],'premises':['ordinary importer has no renames/hardlinks and uses samepath regular predecessors from prior checkpoint base namespace/inode','each first_span is absolute final-file start plus complete canonical chunk payload length','wholefile and incremental/captured construction preserve those coordinates and selected chunks belong to final file','valid contiguous positive-length extents; nonexhausted completed cursor cannot return empty for an in-bounds span','canonical chunk framing is exactly21 bytes; CAS sharing can only reduce unique bytes relative to occurrence growth sum'],'limitations':['This does not assign remaining bytes exclusively to global cap: missing span or other incomplete coverage must be distinguished','No compressed or removable-byte forecast; geometric overlap does not prove useful delta','Invalid/missing spans are excluded from legitimate no-overlap, not silently counted as EOF','Exact per-target historical reason remains unknown; no paths or object IDs logged']}
    (out/'no-overlap-bounds.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps(totals,indent=2))

if __name__=='__main__':
    if sys.argv[1:]==['--self-test']:
        old={'a':['100644',10,'x'],'b':['120000',20,'x'],'c':['100755',30,'x']}
        new={'a':['100755',25,'y'],'b':['100644',90,'y'],'c':['100644',5,'y'],'d':['100644',100,'y']}
        assert growth(old,new)==(15,1)
        assert growth({},new)==(0,0)
        print('oracle growth bound self-test PASS')
    else:main(*map(pathlib.Path,sys.argv[1:]))
