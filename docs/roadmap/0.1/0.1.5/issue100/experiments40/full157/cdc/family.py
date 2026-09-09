"""Full157 unchanged-policy chronological lockfile experiment."""
from pathlib import Path
exec(Path(__file__).with_name('shared.py').read_text())
started=time.perf_counter_ns();original={id:dict(r) for id,r in records.items()}
provenance=json.loads((OUT/'provenance.json').read_text());assert not provenance['missing'] and not provenance['mismatches']
spans={int(k):v for k,v in provenance['spans'].items()};first={k:(v[0],v[1]) for k,v in provenance['first'].items()};sealed_files={int(k):v for k,v in provenance['seals'].items()}
assert len(first)==1347 and len(records)==3221
external=[id for id,r in original.items() if id not in first and r['base'] in first]
assert not external, 'outside family dependencies need broader graph selection'
order=sorted(first,key=lambda id:(records[id]['pack'],records[id]['group'],records[id]['ordinal']))
(OUT/'family-order.json').write_text(json.dumps(dict(protocol_sha256=sha(OUT/'protocol.md'),ids=order),indent=2))
indexes={};index_info=[];results=[];baseline_encode_ns=0
for id in order:
 step,span=first[id];old=original[id]
 saved_graph=records;records=original
 try:raw,_=decode(id)
 finally:records=saved_graph
 # Raw identity of existing base is invariant under simulated physical changes.
 op=decode(old['base'])[0] if old['base'] else b'';reencoded,ns=compress(raw,op);baseline_encode_ns+=ns;assert reencoded==old['frame']
 result=dict(id=id,step=step,span=span,original_kind=old['kind'],original_record_bytes=old['size'],original_base=old['base'])
 if step-1 not in spans:
  assert old['kind']==0;result.update(chosen_kind=old['kind'],chosen_base=old['base'],chosen_record_bytes=old['size'],saved_record_bytes=0,reason='no_preceding_large_file',candidates=[]);results.append(result);continue
 if step-1 not in indexes:
  indexes.clear() # Bound index lifetime to this immediately preceding state only.
  it=time.perf_counter_ns();index={};work=collections.Counter();groups=set()
  for prior in (spans[step-1] if sealed_files[step-1]['length']<=1048576 and len({x['id'] for x in spans[step-1]})<=128 else []):
   bid=prior['id']
   if bid in index:continue
   braw,info=decode(bid);index[bid]=signature(braw);work.update(indexed_raw_bytes=len(braw),closure_raw_bytes=info['raw_closure'],lookups=info['lookups'],encoded_work=info['encoded_work'],decoded_work=info['decoded_work']);groups.update(tuple(g) for g in info['groups'])
  assert len(index)<=128 and work['indexed_raw_bytes']<=1048576
  indexes[step-1]=index;index_info.append(dict(step=step-1,entries=len(index),elapsed_ns=time.perf_counter_ns()-it,unique_physical_groups=len(groups),**dict(work)))
 hints=[]
 for prior in spans[step-1]:
  if prior['start']<span['end'] and prior['end']>span['start'] and prior['id'] not in hints:hints.append(prior['id'])
  if len(hints)==4:break
 if old['kind']:assert hints and old['base']==hints[0]
 sig=signature(raw);matches=[(-len(set(sig)&set(bs)),bid) for bid,bs in indexes[step-1].items() if bid!=id and len(set(sig)&set(bs))>=2];similar=min(matches)[1] if matches else None
 ids=[]
 if hints:ids.append(('first_overlap',hints[0]))
 if similar and similar not in [b for _,b in ids]:ids.append(('similarity',similar))
 full,full_ns=compress(raw);assert decompress(full,len(raw))==raw;bestframe=full;bestbase=None;bestcost=5+len(full);trials=[];budget=collections.Counter(lookups=0,encoded_work=0,decoded_work=0)
 for origin,bid in ids:
  prefix,info=decode(bid);assert records[bid]['pack']<old['pack'];reason=None
  if budget['lookups']+info['lookups']>8 or budget['encoded_work']+info['encoded_work']>524288 or budget['decoded_work']+info['decoded_work']>524288:reason='cumulative_target_read_budget'
  else:
   for k in budget:budget[k]+=info[k]
   if info['depth']>=4 or info['raw_closure']+len(raw)>1048576:reason='depth_or_raw_closure'
  trial=dict(origin=origin,id=bid,work=info)
  if reason:trial['rejection']=reason
  else:
   encoded,ns=compress(raw,prefix);assert decompress(encoded,len(raw),prefix)==raw;cost=37+len(encoded);trial.update(record_bytes=cost,encode_ns=ns)
   if cost<bestcost:bestframe=encoded;bestbase=bid;bestcost=cost
  trials.append(trial)
 records[id]=dict(old,kind=int(bestbase is not None),base=bestbase,frame=bestframe,size=bestcost)
 check,info=decode(id);assert check==raw
 result.update(chosen_kind=records[id]['kind'],chosen_base=bestbase,chosen_record_bytes=bestcost,saved_record_bytes=old['size']-bestcost,full_record_bytes=5+len(full),full_encode_ns=full_ns,first_overlap=hints[0] if hints else None,similarity_candidate=similar,candidates=trials,chosen_closure=info,cumulative_work=dict(budget));results.append(result)
# Reconstruct each original file with the final graph; no original object dropped.
verify=[]
for step,sequence in spans.items():
 t=time.perf_counter_ns();parts=[]
 for span in sequence:parts.append(decode(span['id'])[0])
 data=b''.join(parts);seal=sealed_files[step];assert hashlib.sha256(data).hexdigest()==seal['sha256'];verify.append(dict(step=step,raw_bytes=len(data),chunks=len(sequence),elapsed_ns=time.perf_counter_ns()-t,sha256=seal['sha256']))
assert sha(STORE)==BEFORE
summary=dict(objects=len(results),original_record_bytes=sum(r['original_record_bytes'] for r in results),final_record_bytes=sum(r['chosen_record_bytes'] for r in results),saved_record_bytes=sum(r['saved_record_bytes'] for r in results),improved_targets=sum(r['saved_record_bytes']>0 for r in results),worsened_targets=sum(r['saved_record_bytes']<0 for r in results),unchanged_targets=sum(r['saved_record_bytes']==0 for r in results),original_full_count=sum(r['original_kind']==0 for r in results),final_full_count=sum(r['chosen_kind']==0 for r in results),new_full_count=sum(r['original_kind']!=0 and r['chosen_kind']==0 for r in results),new_full_record_bytes=sum(r['chosen_record_bytes'] for r in results if r['original_kind']!=0 and r['chosen_kind']==0),index_build_ns=sum(r['elapsed_ns'] for r in index_info),index_encoded_work=sum(r['encoded_work'] for r in index_info),index_decoded_work=sum(r['decoded_work'] for r in index_info),index_lookups=sum(r['lookups'] for r in index_info),baseline_encode_ns=baseline_encode_ns,total_elapsed_ns=time.perf_counter_ns()-started)
artifact=dict(scope='Chronological counterfactual physical graph for1347lockfilechunks acrossfull157. Original groups retained for directory-read accounting; no product batch remaining budgets, regenerated packs/SQL or Store allocation claim.',protocol_sha256=sha(OUT/'protocol.md'),script_sha256=sha(Path(__file__)),sample_sha256=sha(OUT/'provenance.json'),store_sha256=BEFORE,summary=summary,indexes=index_info,results=results,verification=verify,selected_graph={id:dict(records[id],frame=records[id]['frame'].hex()) for id in order})
(OUT/'family-result.json').write_text(json.dumps(artifact,indent=2));print(json.dumps(summary,indent=2))
