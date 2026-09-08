#!/usr/bin/env python3
"""Summarize original per-phase smoke receipts; no product or Store access."""
import hashlib,json,pathlib,sys

def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def one(rows,kind):
 x=[r for r in rows if r.get('kind')==kind];assert len(x)==1;return x[0]

def metrics(folder):
 d=json.loads((folder/'performance-result.json').read_text());v=json.loads((folder/'verification-result.json').read_text())
 assert d['status']==v['status']==d['cleanup_status']==v['cleanup_status']=='PASS'
 assert d['container_removed'] and v['container_removed']
 def phases(rows):
  p={}
  for r in rows:
   if r.get('kind')=='storage-smoke-phase':assert r['phase'] not in p;p[r['phase']]=r
  return p
 ready=phases(d['ready']);steps=[]
 for row in d['records']:
  p=phases(row['receipts']);a=one(row['receipts'],'storage-smoke-allocation');physical={}
  for r in p.values():
   for k,x in r.get('physical_storage',{}).items():physical[k]=physical.get(k,0)+x
  steps.append({'index':row['index'],'created':row['created'],'allocation':a,'phases_ns':{k:x['elapsed_ns'] for k,x in p.items()},'public_mutation_commit_ns':sum(x['elapsed_ns'] for k,x in p.items() if k in ['exec','sdk-edit','commit']),'phase_host_cpu_ns':sum(x['host_cpu_ns'] for x in p.values()),'phase_host_read_bytes':sum(x['host_disk_read_bytes'] for x in p.values()),'phase_host_write_bytes':sum(x['host_disk_write_bytes'] for x in p.values()),'physical':physical})
 final=steps[-1]['allocation'];first=one(d['ready'],'storage-smoke-allocation')
 return {'status':'PASS','case':d['case'],'init_ns':ready['init']['elapsed_ns'],'init_allocation':first,'final_allocation':final,'steps':steps,'public_mutation_commit_ns':sum(s['public_mutation_commit_ns'] for s in steps),'phase_host_cpu_ns':sum(s['phase_host_cpu_ns'] for s in steps),'phase_host_read_bytes':sum(s['phase_host_read_bytes'] for s in steps),'phase_host_write_bytes':sum(s['phase_host_write_bytes'] for s in steps),'read_passes_ns':[phases(x)['exec']['elapsed_ns'] for x in d.get('read_passes',[])],'performance_wall_ns':d['wall_ns'],'performance_work_ns':d['work_wall_ns'],'verification_wall_ns':v['wall_ns'],'verified_mappings_count':len(v['records']),'cleanup_status':'PASS','source_performance_sha256':sha(folder/'performance-result.json'),'source_verification_sha256':sha(folder/'verification-result.json')}

def main(runs,out):
 results=[]
 for label in ['deepseek','edits','small']:
  base=runs/f'issue88-s1-baseline-{label}-1';cand=runs/f'issue88-s1-candidate-{label}-1'
  if not cand.exists():continue
  for folder in base.iterdir():
   if not folder.is_dir() or not (folder/'performance-result.json').exists():continue
   b=metrics(folder);c=metrics(cand/folder.name)
   results.append({'case':folder.name,'baseline':b,'candidate':c,'allocation_difference_bytes':c['final_allocation']['store_allocated_bytes']-b['final_allocation']['store_allocated_bytes'],'public_time_difference_ns':c['public_mutation_commit_ns']-b['public_mutation_commit_ns'],'public_time_ratio':{'numerator':c['public_mutation_commit_ns'],'denominator':b['public_mutation_commit_ns']}})
 result={'status':'PASS','scope':'single freshpair development/regression comparison percase; notthree-pairfinalqualification; primaryallocationpreverification','units':'integerbytes/ns/counts, ratiooperands explicit','physical_counter_population':'candidate eligible/trials includeexactinodeleaves pluschunks, baselinechunkonly; noeligiblecountdirectequivalence','cases':results}
 with out.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 for r in results:print(r['case'],r['baseline']['final_allocation']['store_allocated_bytes'],r['candidate']['final_allocation']['store_allocated_bytes'],r['public_time_ratio'])
if __name__=='__main__':main(*map(pathlib.Path,sys.argv[1:]))
