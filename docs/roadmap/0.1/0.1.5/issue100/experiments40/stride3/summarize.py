"""Derive the53-state comparison only from saved receipts and original seals."""
import hashlib,json,statistics
from pathlib import Path
ROOT=Path(__file__).resolve().parent
CASE='deepseek-stride3'
def read(p):return json.loads(p.read_text())
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def timings(result):
 phases={name:[x['elapsed_ns'] for r in result['records'] for x in r['receipts'] if x['kind']=='storage-smoke-phase' and x['phase']==name] for name in ('exec','commit')}
 return {name:dict(count=len(xs),sum_ns=sum(xs),median_ns=statistics.median(xs),min_ns=min(xs),max_ns=max(xs)) for name,xs in phases.items()}
def main():
 run=ROOT/'layerfs';base=run/CASE
 p=read(base/'performance-result.json');v=read(base/'verification-result.json');identity=read(run/'identity.json');g=read(ROOT/'git/results.json')
 assert p['status']==v['status']==g['status']=='PASS'
 assert p['cleanup_status']==v['cleanup_status']==g['cleanup_status']=='PASS'
 assert p['container_removed'] and v['container_removed'] and len(p['records'])==len(v['records'])==53
 assert [r['full157_index'] for r in p['records']]==list(range(1,158,3)) and all(r['created'] for r in p['records'])
 assert all(r['status']=='PASS' for r in v['records'])
 for r in p['records']:
  calls=[x for x in r['receipts'] if x['kind']=='storage-smoke-calls'];assert len(calls)==1 and calls[0]['commit_call_count']==1
  assert calls[0]['sdk_edit_call_count']==0 and calls[0]['exec_process_count']==1
  assert sha(Path(r['oracle']))==r['oracle_sha256']
 alloc=next(x for x in p['closed'] if x['kind']=='storage-smoke-allocation' and x['label']=='after-end')
 last=next(x for x in p['records'][-1]['receipts'] if x['kind']=='storage-smoke-allocation');assert alloc['store_allocated_bytes']==last['store_allocated_bytes'] and alloc['store_apparent_bytes']==last['store_apparent_bytes']
 packed=g['phases']['pack_delta'];assert packed['verification']['commits']==53 and packed['verification']['content']['states']==53
 assert [(r['full157_index'],r['sha'],r['tree']) for r in p['records']]==[(r['full157_index'],r['source_sha'],r['tree']) for r in read(ROOT/'git/mapping.json')]
 paths=sum(r['verified_entries'] for r in v['records']);logical=sum(r['verified_bytes'] for r in v['records'])
 assert paths==306861 and logical==sum(r['logical_bytes'] for r in p['records'])
 assert paths==packed['verification']['content']['verified_entries'] and logical==packed['verification']['content']['verified_bytes']
 full=read(Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full/performance-result.json'))
 current=timings(p);prior=timings(full)
 resources=dict(host_max_observed_rss_bytes=max(x.get('host_rss_bytes',0) for r in p['records'] for x in r['receipts']),host_process_lifetime_peak_rss_bytes=max(x.get('host_lifetime_peak_rss_bytes',0) for r in p['records'] for x in r['receipts']),linux_max_recorded_memory_peak_bytes=max(r['cgroup'].get('memory_peak',0) for r in p['records']),linux_max_recorded_swap_bytes=max(r['cgroup'].get('swap_current',0) for r in p['records']),linux_oom_kills=max(r['cgroup'].get('oom_kill',0) for r in p['records']))
 assert resources['linux_oom_kills']==resources['linux_max_recorded_swap_bytes']==0
 result=dict(resources=resources,status='PASS',scope='Public retained LayerFS53 vs matched Git53; not offline structural format or release qualification',admission_eligible=False,states=53,full157_indices=list(range(1,158,3)),path_states=paths,logical_bytes=logical,layerfs_allocation=alloc,git_packed=packed['storage'],git_loose=g['phases']['loose']['storage'],allocated_gap=alloc['store_allocated_bytes']-packed['storage']['allocated_bytes'],allocated_ratio=alloc['store_allocated_bytes']/packed['storage']['allocated_bytes'],layerfs_timings=current,layerfs_performance_scopes={k:p[k] for k in ('setup_ns','work_wall_ns','cleanup_ns','wall_ns')},layerfs_verification_scopes={k:v[k] for k in ('setup_ns','work_wall_ns','cleanup_ns','wall_ns')},git_construction_ns=g['construction_ns'],git_repack_ns=packed['repack_ns'],git_wall_ns=g['total_wall_ns'],historical_full157=dict(scope='Different workload cardinality and execution date, descriptive iteration-cost comparison only',timings=prior,performance_wall_ns=full['wall_ns'],verification_wall_ns=read(Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full/verification-result.json'))['wall_ns'],commit_sum_ratio_53_over_157=current['commit']['sum_ns']/prior['commit']['sum_ns']),source=identity['source'],host_identity=identity['host_identity'],image_id=identity['image_id'],custody={str(x.relative_to(ROOT)):sha(x) for x in (base/'performance-result.json',base/'verification-result.json',run/'identity.json',ROOT/'git/results.json',ROOT/'git/mapping.json')},script_sha256=sha(Path(__file__)))
 (ROOT/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
