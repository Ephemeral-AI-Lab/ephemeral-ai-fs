from pathlib import Path
import json,runpy
r=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs');helpers=runpy.run_path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue88-experiments/docs/roadmap/0.1/0.1.4/issue88-experiments/summarize_public.py');diag=runpy.run_path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue88-coverage/docs/roadmap/0.1/0.1.4/issue88-delivery/validate_run.py');rows=[]
for label in ['deepseek','edits','small']:
 a=r/f'issue88-SP-control-{label}-1';b=r/f'issue88-SP-candidate-{label}-1';assert json.loads((a/'identity.json').read_text())['fixtures']==json.loads((b/'identity.json').read_text())['fixtures']
 for case in json.loads((a/'performance-summary.json').read_text())['cases']:
  sides=[]
  for folder in [a/case,b/case]:
   m=helpers['metrics'](folder);p=json.loads((folder/'performance-result.json').read_text());v=json.loads((folder/'verification-result.json').read_text());verified={x['index']:x for x in v['records']};assert len(verified)==len(v['records']);assert set(verified)=={x['index'] for x in p['records']}|({0} if case!='deepseek-five' else set());assert all(x['status']=='PASS' for x in verified.values())
   if 0 in verified:assert verified[0]['identity']=='initial'
   for raw,step in zip(p['records'],m['steps']):
    assert raw['identity']==verified[raw['index']]['identity'];step['physical'].update(diag['aggregate'](raw['receipts']))
   sides.append(m)
  x,y=sides;row={'case':case,'control':x,'candidate':y,'allocation_difference_bytes':y['final_allocation']['store_allocated_bytes']-x['final_allocation']['store_allocated_bytes'],'logical_difference_bytes':y['final_allocation']['database_logical_bytes']-x['final_allocation']['database_logical_bytes'],'public_difference_ns':y['public_mutation_commit_ns']-x['public_mutation_commit_ns']};rows.append(row)
  def total(m,k):return sum(s['physical'][k] for s in m['steps'])
  print(case,'allocated',x['final_allocation']['store_allocated_bytes'],y['final_allocation']['store_allocated_bytes'],'logical',x['final_allocation']['database_logical_bytes'],y['final_allocation']['database_logical_bytes'],'publicns',x['public_mutation_commit_ns'],y['public_mutation_commit_ns'],'ratio',y['public_mutation_commit_ns']/x['public_mutation_commit_ns'],'limitedB',total(x,'diag_limited_empty_bytes'),total(y,'diag_limited_empty_bytes'),'grants',total(x,'diag_cursor_grants'),total(y,'diag_cursor_grants'))
  print(' commitns',sum(s['phases_ns'].get('commit',0) for s in x['steps']),sum(s['phases_ns'].get('commit',0) for s in y['steps']),'full/delta',total(x,'diag_new_full_count'),total(y,'diag_new_full_count'),total(x,'diag_new_delta_count'),total(y,'diag_new_delta_count'))
with (r/'issue88-SP-smoke-custody-1/results.json').open('x') as f:json.dump({'status':'PASS','scope':'single fresh C+S1/P developmentpair perapprovedsmoke; correctness/cleanupPASS notblanketperformanceacceptance','units':'integerbytes/ns/counts; gauges maxacrossphases','initial_verification':'edits/small include additional index0 initial state; not discarded or mistaken for extra performance operation','cases':rows},f,indent=2);f.write('\n')
