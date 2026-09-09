import hashlib,json,pathlib,sys
root=pathlib.Path(sys.argv[1]);out=[]
for p in sorted(root.glob('*/perf.jsonl')):
 s=next(x for x in map(json.loads,p.read_text().splitlines()) if x.get('kind')=='sample');rs=s['records']
 def get(k,phase=None):return next((r for r in rs if r.get('kind')==k and (phase is None or r.get('phase')==phase)),{})
 b=get('host-resources','before');a=get('host-resources','after');i=s['identities']
 out.append(dict(case=i['case'],arm=i['source_arm'],elapsed_ns=get('sample-complete').get('pure_call_sum_ns'),user_cpu_ns=a['user_cpu_ns']-b['user_cpu_ns'],system_cpu_ns=a['system_cpu_ns']-b['system_cpu_ns'],lifetime_peak_rss_bytes=a['peak_resident_bytes'],sampled_rss_peak_bytes=get('host-rss-samples')['sampled_peak_bytes'],allocated_bytes=get('store-observation','after-initialize')['allocated_bytes'],input_identity=i['input_identity'],fixture_compatibility=s['preparation']['compatibility'],harness_identity=i['harness_identity'],host=i['host_executor'],image=i['image'],raw=dict(path=str(p),sha256=hashlib.sha256(p.read_bytes()).hexdigest())))
(root/'summary.json').write_text(json.dumps(out,indent=2)+'\n')
for case in sorted({r['case'] for r in out}):
 rows={r['arm']:r for r in out if r['case']==case}
 if len(rows)!=2:continue
 b,a=rows['baseline'],rows['candidate'];assert b['fixture_compatibility']==a['fixture_compatibility'];assert b['harness_identity']==a['harness_identity']
 print(case, 'ms',b['elapsed_ns']/1e6,'->',a['elapsed_ns']/1e6,'change%',100*(a['elapsed_ns']/b['elapsed_ns']-1),'RSS delta',a['lifetime_peak_rss_bytes']-b['lifetime_peak_rss_bytes'],'storage',b['allocated_bytes'],a['allocated_bytes'])
