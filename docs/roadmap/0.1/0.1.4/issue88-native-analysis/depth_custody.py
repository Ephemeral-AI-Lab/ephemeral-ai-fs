#!/usr/bin/env python3
"""Authenticate existing evidence and make two bounded logical copies once."""
import fcntl,hashlib,json,os,pathlib,shutil,subprocess,sys,time,runpy
R=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs')
HERE=pathlib.Path(__file__).resolve().parent

def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def ref(p):return {'path':str(p),'sha256':sha(p)}
def write(p,data):
 with p.open('x') as f:json.dump(data,f,indent=2);f.write('\n')
def quiescent(p):
 assert p.is_file() and not p.is_symlink()
 q=subprocess.run(['lsof','-t','--',str(p)],capture_output=True,text=True,timeout=30)
 assert q.returncode==1 and not q.stdout and not q.stderr,(str(p),q.stdout,q.stderr)
 assert not list(p.parent.glob(p.name+'-*')),'unexpected SQLite sidecar'
def main(out):
 with (pathlib.Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
  fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);started=time.monotonic_ns();out.mkdir()
  assert shutil.disk_usage(R).free>=50*1024**3
  report=R/'issue88-SP-report-1';manifest=report/'manifest.sha256.json';assert sha(manifest)=='d5e220d8afae476ec994caedb47c76d07a37cbc66651bd6ec16b090ba47483ee'
  for name,v in json.loads(manifest.read_text())['files'].items():assert sha(report/name)==v['sha256'],name
  schedule_path=R/'issue88-SP-full157-custody-1/schedule.json';assert sha(schedule_path)==json.loads((report/'manifest.sha256.json').read_text())['files']['schedule.json']['sha256'];schedule=json.loads(schedule_path.read_text())
  old=R/'issue88-report-1';old_manifest=old/'manifest.sha256.json';assert sha(old_manifest)=='57c6e5b4f0c0352d4c57e1e8b2bcb0540aee8d27afeb7e2e4f9a7af6657dfb27'
  old_registry=old/'artifacts.sha256.json';assert sha(old_registry)==json.loads(old_manifest.read_text())['files']['artifacts.sha256.json']['sha256'];reg=json.loads(old_registry.read_text())['artifacts']
  extraction=R/'issue88-content-inputs-2/extraction.sqlite';assert sha(extraction)==reg['issue88-content-inputs-2']['extraction.sqlite']['sha256'];quiescent(extraction)
  es=R/'issue88-content-inputs-2/extraction-summary.json';assert sha(es)==reg['issue88-content-inputs-2']['extraction-summary.json']['sha256']
  results={}
  for arm in schedule['order']:
   name=arm['arm'];run=pathlib.Path(arm['output']);snapshot=R/f'issue88-SP-{name}-snapshot-1/store.sqlite';original=run/'deepseek-full/host-runtime/store.sqlite';quiescent(original);quiescent(snapshot)
   checks=[]
   for phase in ['performance','verification']:
    m=run/f'{phase}-manifest.json';entries=json.loads(m.read_text())
    for rel,digest in entries.items():assert sha(snapshot if phase=='performance' and rel=='deepseek-full/host-runtime/store.sqlite' else run/rel)==digest,(name,phase,rel)
    checks.append({'phase':phase,'manifest':ref(m),'entries':len(entries)})
   inv=R/f'issue88-SP-{name}-census-1/roles-inventory.sqlite';proof=R/f'issue88-SP-{name}-proof-1/inventory-proof.json';expected=json.loads(proof.read_text());assert sha(inv)==expected['inventory_sha256'];assert sha(snapshot)==expected['snapshot_sha256'];quiescent(inv)
   host=pathlib.Path(arm['performance'][arm['performance'].index('--host-binary')+1]);assert sha(host)==arm['host']['binary_sha256']
   source_code='import sys,json;sys.path.insert(0,"benchmark/fs-bench-pro/shared");import runner;print(json.dumps(runner.source_build_args()))'
   current=json.loads(subprocess.check_output(['python3','-c',source_code],cwd=arm['cwd'],text=True))
   for key in ['LAYERFS_SOURCE_SEAL','LAYERFS_PRODUCT_SEAL','WORKLOAD_SOURCE_SHA256']:assert current[key]==arm['host'][key],key
   image=json.loads(subprocess.check_output(['docker','image','inspect',arm['image_id']],text=True))[0]
   assert image['Id']==arm['image_id'];labels=image['Config']['Labels']
   assert labels['dev.layerfs.source-seal']==current['LAYERFS_SOURCE_SEAL'] and labels['dev.layerfs.product-seal']==current['LAYERFS_PRODUCT_SEAL']
   copy=out/'copies'/name/'store.sqlite';copy.parent.mkdir(parents=True)
   with snapshot.open('rb') as src,copy.open('xb') as dst:shutil.copyfileobj(src,dst,1024*1024);dst.flush();os.fsync(dst.fileno())
   assert sha(copy)==sha(snapshot)==expected['snapshot_sha256'];assert copy.stat().st_ino!=snapshot.stat().st_ino
   results[name]={'run':str(run),'source_snapshot':ref(snapshot),'original_store':ref(original),'copy':ref(copy),'inventory':ref(inv),'inventory_proof':ref(proof),'host_binary':ref(host),'producer_host_identity':arm['host'],'current_checkout_source':current,'image_id':image['Id'],'image_labels':labels,'manifests':checks,'copy_method':'independent streamed logical bytes, fsync; no allocation-equivalence claim'}
  qualification=schedule['candidate_build_qualification'];assert sha(pathlib.Path(qualification['path']))==qualification['sha256']
  data={'schema':'issue88-depth-preparation-custody-v1','status':'PASS','arms':results,'schedule':ref(schedule_path),'completed_report_manifest':ref(manifest),'extraction':ref(extraction),'extraction_provenance':ref(old_registry),'extraction_summary':ref(es),'historical_extraction_manifest':ref(old_manifest),'contract':ref(HERE.parent/'issue88-native-design/depth-read-contract-v1.md'),'candidate_build_qualification':qualification,'elapsed_ns':time.monotonic_ns()-started,'scope':'exclusive-lock evidence authentication and independent preverification logical copies; no SQLite/SDK open yet; copying/hashing warms caches','free_bytes_after':shutil.disk_usage(R).free}
  write(out/'custody.json',data);print(json.dumps({'status':'PASS','output':str(out),'elapsed_ns':data['elapsed_ns']}))
if __name__=='__main__':main(pathlib.Path(sys.argv[1]).resolve())
