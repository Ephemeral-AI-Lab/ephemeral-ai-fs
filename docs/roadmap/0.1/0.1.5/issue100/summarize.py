"""Derive full157 comparisons from immutable operation and verification receipts."""
import collections
import csv
import hashlib
import json
import math
from pathlib import Path
import statistics
import sys


def distribution(values):
    return dict(n=len(values),min=min(values),median=statistics.median(values),p95=sorted(values)[math.ceil(.95*len(values))-1],max=max(values),sum=sum(values))


def arm(root, name):
    folder=root/name
    identity=json.loads((folder/'identity.json').read_text())
    case=folder/identity['smoke']
    indices=[r['index'] for r in identity['fixtures'][identity['smoke']]['states']]
    perf=json.loads((case/'performance-result.json').read_text())
    proof=json.loads((case/'verification-result.json').read_text())
    assert perf['status']==proof['status']==perf['cleanup_status']==proof['cleanup_status']=='PASS'
    assert [r['index'] for r in perf['records']]==[r['index'] for r in proof['records']]==indices
    assert [r['identity'] for r in perf['records']]==[r['identity'] for r in proof['records']]
    initial=next(r for r in perf['ready'] if r['kind']=='storage-smoke-allocation')
    rows=[]; physical=collections.defaultdict(collections.Counter); phases=[]; reads=[]
    for s,v in zip(perf['records'],proof['records']):
        ps={r['phase']:r for r in s['receipts'] if r['kind']=='storage-smoke-phase'}
        alloc=next(r for r in s['receipts'] if r['kind']=='storage-smoke-allocation')
        calls=next(r for r in s['receipts'] if r['kind']=='storage-smoke-calls')
        assert calls['exec_process_count']==calls['commit_call_count']==1 and calls['sdk_edit_call_count']==0
        assert v['status']=='PASS'
        rows.append(dict(arm=name,step=s['index'],source_sha=s['sha'],created=s['created'],commit_id=s['identity'],
            exec_ns=ps['exec']['elapsed_ns'],commit_ns=ps['commit']['elapsed_ns'],paired_ns=ps['exec']['elapsed_ns']+ps['commit']['elapsed_ns'],
            allocated_bytes=alloc['store_allocated_bytes'],logical_bytes=alloc['database_logical_bytes'],
            growth_bytes=alloc['store_allocated_bytes']-initial['store_allocated_bytes'],transfer_ns=s['transfer_ns'],
            exec_cpu_ns=ps['exec']['host_cpu_ns'],commit_cpu_ns=ps['commit']['host_cpu_ns'],
            verification_step_wall_ns=v['step_wall_ns'],verified_entries=v['verified_entries'],verified_bytes=v['verified_bytes']))
        for phase,r in ps.items():
            physical[phase].update({k:v for k,v in r['physical_storage'].items() if isinstance(v,int)})
            phases.append(r)
        reads.extend(r for r in v['receipts'] if r['kind']=='storage-smoke-phase')
    final=next(r for r in perf['records'][-1]['receipts'] if r['kind']=='storage-smoke-allocation')
    resources={}
    for label,data,ops in [('performance',perf,phases),('verification',proof,reads)]:
        resources[label]=dict(host_lifetime_peak_rss_bytes=max(r['host_lifetime_peak_rss_bytes'] for r in ops),
            host_max_boundary_rss_bytes=max(r['host_rss_bytes'] for r in ops),host_cpu_ns=sum(r['host_cpu_ns'] for r in ops),
            cgroup_max_boundary={k:max(s['cgroup'].get(k,0) for s in data['records']) for k in data['records'][0]['cgroup'] if isinstance(data['records'][0]['cgroup'][k],int)},
            cgroup_memory_stat_max_boundary={k:max(s['cgroup']['memory_stat_bytes'].get(k,0) for s in data['records']) for k in data['records'][0]['cgroup']['memory_stat_bytes']},
            spool_max_boundary_bytes=max(s['spool_disk']['allocated_bytes'] for s in data['records']),
            staging_max_boundary_bytes=max(s['container_staging_allocated_bytes'] for s in data['records']))
    metrics=dict(initial_allocated_bytes=initial['store_allocated_bytes'],final_allocated_bytes=final['store_allocated_bytes'],
        retained_growth_bytes=final['store_allocated_bytes']-initial['store_allocated_bytes'],
        initial_logical_bytes=initial['database_logical_bytes'],final_logical_bytes=final['database_logical_bytes'],
        performance_wall_ns=perf['wall_ns'],performance_work_wall_ns=perf['work_wall_ns'],verification_wall_ns=proof['wall_ns'],verification_work_wall_ns=proof['work_wall_ns'],
        setup_ns=perf['setup_ns'],verification_setup_ns=proof['setup_ns'],cleanup_ns=perf['cleanup_ns'],verification_cleanup_ns=proof['cleanup_ns'],
        preparation_ns=identity['preparation_ns'],verification_preparation_ns=json.loads((folder/'verification-summary.json').read_text())['preparation_ns'],
        transfer_ns=sum(r['transfer_ns'] for r in rows))
    summary=dict(name=name,metrics=metrics,distributions={k:distribution([r[k] for r in rows]) for k in ('exec_ns','commit_ns','paired_ns','verification_step_wall_ns')},historical_read_exec_ns=distribution([r['elapsed_ns'] for r in reads if r['phase']=='exec']),
        slowest_paired=sorted(rows,key=lambda r:r['paired_ns'],reverse=True)[:8],resources=resources,physical_counters=dict(physical),
        created=sum(s['created'] for s in perf['records']),up_to_date=sum(not s['created'] for s in perf['records']),
        verified_states=len(proof['records']),verified_entries=sum(r['verified_entries'] for r in proof['records']),verified_bytes=sum(r['verified_bytes'] for r in proof['records']),
        cleanup=dict(performance=perf['cleanup_status'],verification=proof['cleanup_status']),source=identity['source'],host=identity['host_identity'],image=identity['image_id'],
        census=json.loads((folder/'census.json').read_text()),
        fixture_digest=hashlib.sha256(json.dumps(identity['fixtures'],sort_keys=True).encode()).hexdigest(),
        contract_digest=identity['full_run_contract_sha256'] or identity['contract_sha256'])
    return summary,rows


root=Path(sys.argv[1]); names=sys.argv[2:]; summaries=[];rows=[]
for name in names:
    s,r=arm(root,name);summaries.append(s);rows.extend(r)
assert len({s['fixture_digest'] for s in summaries})==len({s['contract_digest'] for s in summaries})==1
comparisons={}
for s in summaries[1:]:
    comparisons[s['name']]={k:dict(difference=v-summaries[0]['metrics'][k],percent=(v/summaries[0]['metrics'][k]-1)*100 if summaries[0]['metrics'][k] else None) for k,v in s['metrics'].items()}
    for phase,d in s['distributions'].items():
        for statistic in ('median','p95','sum'):
            b=summaries[0]['distributions'][phase][statistic]
            comparisons[s['name']][phase+'_'+statistic]=dict(difference=d[statistic]-b,percent=(d[statistic]/b-1)*100)
with (root/('comparison-'+'-'.join(names)+'.json')).open('x') as f:json.dump(dict(arms=summaries,comparisons=comparisons,admission_eligible=False),f,indent=2)
with (root/('comparison-'+'-'.join(names)+'.csv')).open('x') as f:
    writer=csv.DictWriter(f,fieldnames=list(rows[0]));writer.writeheader();writer.writerows(rows)
print(json.dumps({s['name']:dict(metrics=s['metrics'],distributions=s['distributions'],created=s['created'],up_to_date=s['up_to_date']) for s in summaries},indent=2))
