"""Conditional one-candidate previous-file minhash diagnostic; fixed12 targets."""
from pathlib import Path
# Reuse verified parser, native codec, canonical hash and decoder without running
# overlap selection/trials or overwriting their artifacts.
helper=Path(__file__).with_name('experiment.py')
exec(helper.read_text().split('fixture=json.loads')[0])
sample=json.loads((OUT/'sample.json').read_text());samples=sample['samples'];mask=(1<<64)-1
# Exact existing small_candidates.rs signature, already used by retained-delta-pairs.py.
def signature(raw):
 result=[];rolling=0;high=pow(257,15,1<<64)
 for i,b in enumerate(raw):
  if i>=16:rolling=(rolling-raw[i-16]*high)&mask
  rolling=(rolling*257+b)&mask
  if i<15:continue
  h=rolling;h=((h^(h>>30))*0xbf58476d1ce4e5b9)&mask;h=((h^(h>>27))*0x94d049bb133111eb)&mask;h=h^(h>>31)
  if h!=mask and (len(result)<8 or h<result[-1]) and h not in result:result=sorted(result+[h])[:8]
 return result
assert signature(b'')==[] and len(signature(b'0123456789abcdef'))==1
assert set(signature(b'prefix'+b'abcdefghijklmno'*20)).intersection(signature(b'abcdefghijklmno'*20))
indexes={};index_results=[]
for step in sorted({s['step']-1 for s in samples}):
 seal=sample['fixture_seals'][str(step)];path=Path(seal['path']);assert sha(path)==seal['sha256'];data=path.read_bytes();assert len(data)<=1048576
 expected={};pos=0
 for size in lengths(data):
  raw=data[pos:pos+size];pos+=size;expected[object_id(raw)]=raw
 assert pos==len(data) and len(expected)<=128
 started=time.perf_counter_ns();index={};work=collections.Counter();groups=set()
 for id,original in expected.items():
  raw,info=decode(id);assert raw==original;index[id]=signature(raw)
  work.update(indexed_raw_bytes=len(raw),closure_raw_bytes=info['raw_closure'],lookups=info['lookups'],encoded_work=info['encoded_work'],decoded_work=info['decoded_work']);groups.update(tuple(g) for g in info['groups'])
 indexes[step]=index;index_results.append(dict(step=step,entries=len(index),elapsed_ns=time.perf_counter_ns()-started,unique_physical_groups=len(groups),groups=sorted(groups),**dict(work)))
results=[]
for s in samples:
 raw,_=decode(s['id']);started=time.perf_counter_ns();sig=signature(raw);matches=[]
 for id,base_sig in indexes[s['step']-1].items():
  overlap=len(set(sig)&set(base_sig))
  if id!=s['id'] and overlap>=2:matches.append((-overlap,id))
 chosen=min(matches) if matches else None;selection_ns=time.perf_counter_ns()-started
 result=dict(id=s['id'],step=s['step'],current_record_bytes=s['current_record_bytes'],selection_ns=selection_ns,saved_record_bytes=0,matching_signature_candidates=len(matches))
 if chosen:
  negative,id=chosen;prefix,info=decode(id);assert records[id]['pack']<records[s['id']]['pack'];result.update(candidate_id=id,shared_fingerprints=-negative,already_overlap_hint=id in s['hints'],base_work=info)
  if info['depth']>=4 or info['raw_closure']+len(raw)>1048576:result['rejection']='depth_or_raw_closure'
  elif info['lookups']>8 or info['encoded_work']>524288 or info['decoded_work']>524288:result['rejection']='target_read_budget'
  else:
   encoded,ns=compress(raw,prefix);assert decompress(encoded,len(raw),prefix)==raw
   cost=37+len(encoded);result.update(alternate_record_bytes=cost,encode_ns=ns,saved_record_bytes=max(0,s['current_record_bytes']-cost))
 else:result['rejection']='no_two_fingerprint_candidate'
 results.append(result)
summary=dict(targets=len(results),current_record_bytes=sum(r['current_record_bytes'] for r in results),saved_record_bytes=sum(r['saved_record_bytes'] for r in results),improved_targets=sum(r['saved_record_bytes']>0 for r in results),candidates=sum('candidate_id' in r for r in results),already_overlap_candidates=sum(r.get('already_overlap_hint',False) for r in results),encoded_candidates=sum('alternate_record_bytes' in r for r in results),index_steps=len(index_results),index_build_ns=sum(r['elapsed_ns'] for r in index_results),index_decoded_work=sum(r['decoded_work'] for r in index_results),index_encoded_work=sum(r['encoded_work'] for r in index_results),index_lookups=sum(r['lookups'] for r in index_results))
assert sha(STORE)==BEFORE
result=dict(scope='One fixed previous-file minhash policy; SAME frozen12 targets. Diagnostic exact existing graph; no product integration/physical rewrite/wholeStore or complete family savings claim.',protocol_sha256=sha(OUT/'protocol2.md'),sample_sha256=sha(OUT/'sample.json'),store_sha256=BEFORE,script_sha256=sha(Path(__file__)),helper_sha256=sha(helper),summary=summary,indexes=index_results,results=results)
(OUT/'similarity-result.json').write_text(json.dumps(result,indent=2));print(json.dumps(summary,indent=2))
