from pathlib import Path
import json,subprocess,time,os,fcntl,hashlib
r=Path(__file__).parent
root=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-workspace-admission')
start=time.monotonic()
while not (r/'terminal-benchmark-result.json').exists():
 if time.monotonic()-start>7200:raise SystemExit('benchmark completion receipt not available after 2h')
 time.sleep(1)
assert json.loads((r/'terminal-benchmark-result.json').read_text())['returncode']==0,'benchmark failed; do not continue'
image=(r/'final-image-id.txt').read_text().strip()
receipts=[]
def phase(name,argv):
 (r/(name+'-command.json')).write_text(json.dumps({'argv':argv,'cwd':str(root)},indent=2))
 print('START '+name,flush=True);started=time.monotonic_ns()
 with (r/(name+'.log')).open('x') as log:p=subprocess.run(argv,cwd=root,stdout=log,stderr=subprocess.STDOUT)
 row={'name':name,'returncode':p.returncode,'wall_ns':time.monotonic_ns()-started};receipts.append(row);(r/'remaining-pipeline.json').write_text(json.dumps(receipts,indent=2));print(row,flush=True)
 if p.returncode:raise SystemExit(p.returncode)
prep=r/'terminal-preparation-02'
phase('full157-prepare',['python3',str(prep/'prepare.py'),'--source',str(root),'--host-binary',str(root/'target/release/fs-benchmark-pro'),'--image',image,'--build-qualification',str(r/'terminal-build-qualification.json'),'--check-qualification',str(r/'terminal-final-check-qualification.json'),'--terminal-declaration',str(r/'terminal-declaration.json'),'--census-binary','/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91-runs/r26-full157-final/custody/issue88-native-roles','--census-applicability',str(prep/'census-reuse-applicability.json')])
phase('terminal-supplementals',['python3',str(r/'supplementals.py')])
phase('terminal-full157',['python3',str(prep/'run_frozen.py'),str(prep/'schedule.frozen.json'),'--execute'])
with open(os.environ['TMPDIR']+'/layerfs-infra-measurement.lock','a') as lock:
 fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 phase('terminal-venv-build',['cargo','+1.85.1','build','--release','--locked','--offline','--manifest-path',str(root/'docs/roadmap/0.1/0.1.4/issue71-venv-acceptance/probe/Cargo.toml'),'-j2'])
# Reuse the exact full-venv harness, changing only source/image applicability.
here=root/'docs/roadmap/0.1/0.1.4/issue71-venv-acceptance'
s=(here/'run.py').read_text().replace('here = Path(__file__).resolve().parent','here = Path('+repr(str(here))+')')
s=s.replace('plan = json.loads((here / "plan.json").read_text())','plan = json.loads((here / "plan.json").read_text())\nplan["product_commit"] = "9cfb4be477116646258ea0621280ed13b1824c6d"\nplan["image"] = '+repr(image))
(r/'terminal-venv-run.py').write_text(s)
phase('terminal-venv',['python3',str(r/'terminal-venv-run.py'),str(r/'terminal-venv')])
print('ALL REMAINING QUALIFICATION COMMANDS PASS',flush=True)
