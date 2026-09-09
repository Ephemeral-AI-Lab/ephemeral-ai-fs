import hashlib,json,pathlib,subprocess,sys,time
root=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue95');base=pathlib.Path(__file__).parent
control='/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91-runs/r26-build-host-02/fs-benchmark-pro'
control_image='sha256:8e74a58284a6e8c22ac9cbf60231f69217c3cae31c231ef17b0afd81726b8249'
candidate=str(root/'target/release/fs-benchmark-pro');candidate_image=sys.argv[1]
run=base/sys.argv[2];run.mkdir()
rows=[]
for family,case in [('dedup_cross_file','dedup-cross-file-identical-500'),('dedup_cdc_locality','dedup-cdc-overwrite-500'),('dedup_cross_file','dedup-cross-file-unique-500')]:
 for arm,binary,image in [('baseline',control,control_image),('candidate',candidate,candidate_image)]:
  out=run/(case+'-'+arm)
  cmd=['python3',str(root/'benchmark/fs-bench-pro/shared/runner.py'),'--family',family,'--case',case,'--seed','1','--setup','fresh','--host-binary',binary,'--image',image,'--source-arm',arm,'--perf-fast','--collection-mode','--product-timeout','300','--timeout','310','--setup-timeout','600','--output',str(out)]
  (run/(case+'-'+arm+'-command.json')).write_text(json.dumps(cmd,indent=2))
  start=time.monotonic_ns()
  with (run/(case+'-'+arm+'.log')).open('w') as f:r=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT)
  rows.append(dict(case=case,arm=arm,returncode=r.returncode,wall_ns=time.monotonic_ns()-start))
  (run/'ledger.json').write_text(json.dumps(rows,indent=2))
  print(case,arm,r.returncode,flush=True)
  if r.returncode:sys.exit(r.returncode)
