"""Invoke the existing collector for the prospectively ordered R6 cells."""
import argparse, importlib.util, json, os, pathlib, subprocess, sys
ROOT=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91-runs/source-g2')
OUT=ROOT.parent/'g2-profile-100k-01'
sys.path.insert(0,str(ROOT/'benchmark/fs-bench-pro'))
spec=importlib.util.spec_from_file_location('collect', ROOT/'benchmark/fs-bench-pro/issue54_collect.py')
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
image_tag=(ROOT.parent/'build-g2/image.log').read_text().strip().splitlines()[-1]
image=json.loads(subprocess.check_output(['docker','image','inspect',image_tag]))[0]['Id']
args=argparse.Namespace(image=image,host_binary=str(ROOT/'target/release/fs-benchmark-pro'),listed_families={},
 campaign_spec=json.loads((ROOT/'docs/roadmap/0.1/0.1.4/issue91-campaign/declaration.json').read_text()),
 proof_preparation_declaration=ROOT/'docs/roadmap/0.1/0.1.4/issue91-campaign/verification-preparation-r4.json')
args.proof_preparation_spec=json.loads(args.proof_preparation_declaration.read_text())
cases=[('init_namespace','namespace-100000')]
OUT.mkdir()
identity=json.loads(pathlib.Path(args.host_binary+'.identity.json').read_text())
assert identity['LAYERFS_SOURCE_DIRTY']=='false'
assert subprocess.check_output(['git','status','--porcelain'],cwd=ROOT)==b''
(OUT/'declaration.json').write_text(json.dumps(dict(source=str(ROOT),host=identity,image=image,cases=cases,
 seed=1,diagnostic_only=True,amendment='c2dbf4cc9',initialization_diagnostic_nonce='91',
 historical_baseline='9f5a641d223606c45e5e6aa8a20094c12f9139a1',
 command_templates='issue54_collect.collect_row / verify_row / prepare_proof, unchanged300/310/600 and45/59 budgets'),indent=2))
os.environ['LAYERFS_INITIALIZATION_DIAGNOSTIC_NONCE']='91'
performance=[];proofs=[]
import threading,time
def capture():
 end=time.monotonic()+600
 while time.monotonic()<end:
  lines=subprocess.check_output(['ps','-axo','pid,command'],text=True).splitlines()
  matched=[x.strip() for x in lines if str(ROOT/'target/release/fs-benchmark-pro')+' infra-run init_namespace namespace-100000 1 performance' in x]
  # The registered runner uses perform, not a wrapper/fixture command.
  if not matched:
   matched=[x.strip() for x in lines if str(ROOT/'target/release/fs-benchmark-pro')+' infra-run init_namespace namespace-100000 1 perform' in x]
  if matched:
   (OUT/'profile-process.txt').write_text('\n'.join(matched))
   time.sleep(4)
   pid=matched[0].split()[0];cmd=['sample',pid,'5','1','-file',str(OUT/'profile.txt')]
   (OUT/'profile-command.json').write_text(json.dumps(cmd))
   r=subprocess.run(cmd,capture_output=True,text=True)
   (OUT/'profile-result.json').write_text(json.dumps(dict(returncode=r.returncode,stdout=r.stdout,stderr=r.stderr)))
   return
  time.sleep(.05)
 (OUT/'profile-result.json').write_text(json.dumps(dict(error='target not observed')))
monitor=threading.Thread(target=capture,daemon=True);monitor.start()
for family,case in cases:
 rows,listed=c.list_family(args,family);args.listed_families[family]=listed
 row=next(r for r in rows if r['scenario_id']==case)
 result,_=c.collect_row(args,family,row,OUT)
 performance.append(result);c._write(OUT/'performance-ledger.json',performance)
 print('PERF',case,result.get('status'),result.get('elapsed_ns'),flush=True)
 if result.get('status')!='PASS': sys.exit(1)
 proof=c.verify_row(args,family,row,OUT,result['identities'])
 proofs.append(proof);c._write(OUT/'verification-ledger.json',proofs)
 print('PROOF',case,proof.get('status'),flush=True)
 if proof.get('status')!='PASS': sys.exit(1)

monitor.join(timeout=5)
