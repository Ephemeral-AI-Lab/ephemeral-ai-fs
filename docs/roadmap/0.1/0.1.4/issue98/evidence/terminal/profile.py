from pathlib import Path
import os,fcntl,subprocess,time,re,json
r=Path(__file__).parent
with open(os.environ['TMPDIR']+'/layerfs-infra-measurement.lock','a') as lock:
 fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 out=r/'profile-01';out.mkdir()
 argv=[str(r/'candidate-01-proof'),'workspace','/Users/yifanxu/Ephemeral-AI-Lab/torch-layerfs-init-20260908/implementation-evidence/venv.tar',str(out/'store'),'sha256:7679d908d50aa7cac722d0e5fe9f3c5dd1a8e54a1cb3f1728c97a26b26200974']
 (out/'command.json').write_text(json.dumps(argv))
 with (out/'run.log').open('x') as log:
  p=subprocess.Popen(argv,stdout=log,stderr=subprocess.STDOUT)
  start=time.monotonic()
  while p.poll() is None and time.monotonic()-start<120:
   if 'exec_seconds=' in (out/'run.log').read_text():break
   time.sleep(.05)
  assert p.poll() is None
  time.sleep(2.4)
  sample=subprocess.run(['/usr/bin/sample',str(p.pid),'3','1','-file',str(out/'sample.txt')],capture_output=True,text=True)
  (out/'sample-command.log').write_text(sample.stdout+sample.stderr)
  assert p.wait(timeout=480)==0
 branch=re.search(r'^branch=(.+)$',(out/'run.log').read_text(),re.M)[1]
 with (out/'verify.log').open('x') as log:subprocess.run([str(r/'candidate-01-proof'),'verify','/Users/yifanxu/Ephemeral-AI-Lab/torch-uv/.venv',str(out/'store/store.sqlite')],env=dict(os.environ,VERIFY_BRANCH=branch),stdout=log,stderr=subprocess.STDOUT,check=True,timeout=180)
 print('profile and independent verification PASS')
