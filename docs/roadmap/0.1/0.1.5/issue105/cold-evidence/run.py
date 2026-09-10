import datetime,fcntl,hashlib,json,os,subprocess,sys,time,threading
from pathlib import Path
out=Path(__file__).parent;root=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb');sys.path.insert(0,str(root/'benchmark/fs-bench-pro/shared'));import runner
jobs=int(sys.argv[1]);target=out/f'target-j{jobs}';assert not target.exists()
argv=['/usr/bin/time','-l','cargo','+1.85.1','build','--locked','--release',f'-j{jobs}','-p','fs-benchmark-pro','-p','layerfs-layerstack-store','--bins','--target-dir',str(target),'--timings']
lock=Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock'
with lock.open('a') as f:
 fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
 before=runner.source_build_args();stop=threading.Event()
 def progress():
  while not stop.wait(15):print(f'PROGRESS j{jobs} fresh build running',flush=True)
 print('START',json.dumps(argv),flush=True)
 with (out/f'j{jobs}.log').open('x') as log:
  utc=datetime.datetime.now(datetime.timezone.utc).isoformat();start=time.monotonic_ns()
  p=subprocess.Popen(argv,cwd=root,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);t=threading.Thread(target=progress,daemon=True);t.start()
  for line in p.stdout:print(line,end='',flush=True);log.write(line);log.flush()
  rc=p.wait();wall=time.monotonic_ns()-start;end=datetime.datetime.now(datetime.timezone.utc).isoformat();stop.set();t.join()
 record={'argv':argv,'cwd':str(root),'start_utc':utc,'end_utc':end,'wall_ns':wall,'returncode':rc,'source':before}
 with (out/'attempts.jsonl').open('a') as a:a.write(json.dumps(record)+'\n')
 assert rc==0
 binary=target/'release/fs-benchmark-pro';compactor=target/'release/layerfs-store-compact'
 sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
 probe_start=time.monotonic_ns();schema=runner.verify_linked_schema(binary,10);fmt=runner.verify_integrated_format(binary);probe_ns=time.monotonic_ns()-probe_start
 assert runner.source_build_args()==before
 identity={**record,'binary_sha256':sha(binary),'compactor_sha256':sha(compactor),'existing_host_sha256':sha(root/'target/release/fs-benchmark-pro'),'schema':schema,'format':fmt,'probes_wall_ns':probe_ns}
 (out/f'j{jobs}-qualification.json').write_text(json.dumps(identity,indent=2))
 print('END',json.dumps({k:v for k,v in identity.items() if k not in ('source','format')}),flush=True)
