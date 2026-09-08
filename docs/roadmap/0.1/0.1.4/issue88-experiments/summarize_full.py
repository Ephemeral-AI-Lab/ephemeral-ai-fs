#!/usr/bin/env python3
"""Full157 comparison from existing receipts, never Store access."""
import json,pathlib,sys
from summarize_public import metrics

def main(runs,out):
 def raw(name):
  folder=runs/name/'deepseek-full';p=json.loads((folder/'performance-result.json').read_text());v=json.loads((folder/'verification-result.json').read_text())
  assert len(p['records'])==len(v['records'])==157
  for i,(a,b) in enumerate(zip(p['records'],v['records']),1):
   assert a['index']==b['index']==i and a['identity']==b['identity'] and b['status']=='PASS'
   assert any(x.get('kind')=='storage-smoke-verified-read' and x['identity']==a['identity'] for x in b['receipts'])
  return p,v
 bp,bv=raw('full157-m45-1');cp,cv=raw('issue88-s1-full157-1')
  # Existing per-run mappings differ; frozen input/tree/oracle identities must match.
 for a,b in zip(bp['records'],cp['records']):
  for k in ['index','ordinal','sha','tree','manifest_sha256','oracle_sha256','input_seal','files','logical_bytes']:assert a[k]==b[k],(a['index'],k)
 old=metrics(runs/'full157-m45-1/deepseek-full');new=metrics(runs/'issue88-s1-full157-1/deepseek-full')
 a=old['final_allocation']['store_allocated_bytes'];b=new['final_allocation']['store_allocated_bytes']
 def counters(d):
  result={}
  for s in d['steps']:
   for k,v in s['physical'].items():result[k]=result.get(k,0)+v
  return result
 result={'status':'PASS','comparison':'historicalacceptedfull157vsnewpublicS1; sameretained157oraclepopulation, notfreshpairedtiming; initialinode/randomcanonicaldifferencespossible','units':'integerbytes/ns/counts, ratiosasoperands','baseline':old,'candidate':new,'allocation_reduction_bytes':a-b,'allocation_reduction_ratio':{'numerator':a-b,'denominator':a},'target_allocated_bytes':134221004,'target_met':b<=134221004,'baseline_counters':counters(old),'candidate_counters':counters(new),'counter_qualification':'S1eligible/trials expandedtoinodeleaves+chunks; admittedDELTAcountdoesnotaloneidentifyrole; no mixedpopulationlostopportunityclaim'}
 result['all157_mapping_and_input_checks']='PASS'
 result['verified_bytes']={'baseline':sum(r['verified_bytes'] for r in bv['records']),'candidate':sum(r['verified_bytes'] for r in cv['records'])}
 assert result['verified_bytes']['baseline']==result['verified_bytes']['candidate']
 result['phase_scopes']={'performance_wall_ns':'case wall from performance-result, excludes enclosing preparation','performance_work_ns':'case work wall','verification_wall_ns':'verification case wall','physical_counters':'sum per-checkpoint phases only; Init excluded'}
 result['invocations']={n:{m:json.loads((runs/n/(m+'-summary.json')).read_text()) for m in ['performance','verification']} for n in ['full157-m45-1','issue88-s1-full157-1']}
 with out.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps({'baseline_allocated':a,'candidate_allocated':b,'difference':a-b,'baseline_public_ns':old['public_mutation_commit_ns'],'candidate_public_ns':new['public_mutation_commit_ns'],'target_met':result['target_met']}))
if __name__=='__main__':main(*map(pathlib.Path,sys.argv[1:]))
