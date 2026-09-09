"""Read-only corruption checks and worst measured native slice read amplification."""
import collections,hashlib,json,pathlib,struct,time
import reader
HERE=pathlib.Path(__file__).resolve().parent;STORE=HERE/'candidate.sqlite'
base=reader.StoreReader(STORE);files=[];adapters=[]
for i,(p,g,r,n) in base.loc.items():
 data=base._pack(p)
 if data[:8]!=reader.MAGIC:continue
 v,body,count=base._physical(i)
 if body[0] in (0,1,2,3):files.append((i,body,n))
 elif body[0]==4:adapters.append((i,body,n))
full=next(x for x in files if x[1][0]==0);delta=next(x for x in files if x[1][0]==1);adapter=adapters[0]
passed=[]
def reject(name,identity,mutate):
 r=reader.StoreReader(STORE);physical=r._physical
 def changed(i):
  v,b,count=physical(i)
  return (v,mutate(b),count) if i==identity else (v,b,count)
 r._physical=changed
 try:r.read_canonical(identity)
 except (ValueError,struct.error):passed.append(name)
 else:raise AssertionError('corruption accepted: '+name)
 finally:r.close()
reject('frame checksum',full[0],lambda b:b[:-1]+bytes([b[-1]^1]))
reject('file self cycle',delta[0],lambda b:b[:1]+delta[0]+b[33:])
reject('slice missing owner',adapter[0],lambda b:b[:1]+b'\xff'*32+b[33:])
reject('slice wrong owner role',adapter[0],lambda b:b[:1]+adapter[0]+b[33:])
reject('slice extent overflow',adapter[0],lambda b:b[:33]+struct.pack('<II',2**32-1,adapter[2]-21))
# Rank the actual adapter->owner graph by raw decoding/output ratio, not whole-file size alone.
closure={}
for ident,body,n in adapters:
 owner=body[1:33]
 if owner in closure:continue
 node=owner;seen=set();raw=canonical=encoded=0;depth=0
 while True:
  assert node not in seen;seen.add(node);v,r,count=base._physical(node);assert v==107 and r[0] in (0,1,2,3)
  cn=base.loc[node][3];raw+=cn-(23 if r[0] in (0,1) else 21);canonical+=cn;encoded+=len(r)
  if r[0] in (0,2):break
  node=r[1:33];depth+=1
 closure[owner]=(raw,canonical,encoded,depth)
worst=max(adapters,key=lambda x:(closure[x[1][1:33]][0]/(x[2]-21),x[0]))
maxbytes=max(adapters,key=lambda x:(closure[x[1][1:33]][1],x[0]));base.close()
measurements=[]
for label,(ident,body,n) in [('largest_decode_amplification',worst),('largest_owner_graph',maxbytes)]:
 r=reader.StoreReader(STORE);started=time.perf_counter_ns();out=r.read_canonical(ident);elapsed=time.perf_counter_ns()-started;assert len(out)==n
 raw,can,enc,depth=closure[body[1:33]];assert r.metrics['content_canonical_decoded_bytes']==can
 measurements.append(dict(selection=label,chunk_id=ident.hex(),owner_id=body[1:33].hex(),chunk_raw_bytes=n-21,owner_raw_bytes=r.loc[body[1:33]][3]-21,file_graph_raw_decoded_bytes=raw,file_graph_canonical_decoded_bytes=can,file_graph_encoded_record_bytes=enc,adapter_record_bytes=len(body),file_graph_edges=depth,total_dependency_edges=depth+1,raw_decode_amplification=raw/(n-21),cold_reader_elapsed_ns=elapsed,reader_metrics=dict(r.metrics)))
 r.close()
result=dict(status='PASS',negative_checks=passed,all_adapter_count=len(adapters),distinct_owners=len(closure),scope='All adapter graphs ranked; two worst selections actually read with empty per-reader caches. OS cache uncontrolled. Diagnostic elapsed time is not product latency.',measurements=measurements)
(HERE/'reader-checks.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
