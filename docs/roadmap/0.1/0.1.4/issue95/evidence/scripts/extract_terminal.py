import csv,hashlib,json,pathlib,re
root=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue95-runs/terminal-benchmark/performance')
out=pathlib.Path(__file__).parent
rows=[]
for family in ['dedup_cdc_locality','dedup_cross_file']:
 for p in sorted((root/family).glob('*/perf.jsonl')):
  s=next(x for x in map(json.loads,p.read_text().splitlines()) if x.get('kind')=='sample');rs=s['records'];d={};producers=[]
  for r in rs:
   t=r.get('details','')
   if t.startswith('layerfs-initialization-'):
    vals={k:int(v) for k,v in re.findall(r'(\w+)=(\d+)(?= |$)',t)}
    if t.startswith('layerfs-initialization-producer'): producers.append(vals)
    else:d.update(vals)
  get=lambda kind,phase=None:next((r for r in rs if r.get('kind')==kind and (phase is None or r.get('phase')==phase)),{})
  before=get('host-resources','before');after=get('host-resources','after');rss=get('host-rss-samples')
  row=dict(family=family,case=p.parent.name,elapsed_ns=get('sample-complete').get('pure_call_sum_ns'))
  for k in ['pending_duplicate_objects','pending_duplicate_bytes','cross_batch_skipped_objects','cross_batch_skipped_bytes','conflict_read_ns','sql_commit_ns','total_count','sql_returned_ids','slab_consumer_idle_ns','slab_send_blocked_ns','direct_pipeline_wall_ns']:row[k]=d.get(k)
  row.update(producer_blocked_max_ns=max((r['blocked_ns'] for r in producers),default=None),user_cpu_ns=after.get('user_cpu_ns',0)-before.get('user_cpu_ns',0),system_cpu_ns=after.get('system_cpu_ns',0)-before.get('system_cpu_ns',0),lifetime_peak_rss_bytes=after.get('peak_resident_bytes'),sampled_rss_peak_bytes=rss.get('sampled_peak_bytes'),store_allocated_bytes=get('store-observation','after-initialize').get('allocated_bytes'),raw_path=str(p),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
  rows.append(row)
assert len(rows)==30
with (out/'terminal-diagnostics.csv').open('x') as f:
 w=csv.DictWriter(f,fieldnames=rows[0]);w.writeheader();w.writerows(rows)
(out/'terminal-diagnostics.json').write_text(json.dumps(rows,indent=2)+'\n')
for r in rows:print(r['case'],round(r['elapsed_ns']/1e6,2),r['cross_batch_skipped_objects'],round(r['conflict_read_ns']/1e6,2),round(r['sql_commit_ns']/1e6,2),r['total_count'])
