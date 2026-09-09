from pathlib import Path
import fcntl,os,subprocess,json,re,time,hashlib,signal
r=Path(__file__).parent
out=r/"pair-04"
assert not out.exists()
control=Path("/Users/yifanxu/Ephemeral-AI-Lab/layerfs-v014-venv-acceptance/qualified-proof-binary")
image="sha256:7679d908d50aa7cac722d0e5fe9f3c5dd1a8e54a1cb3f1728c97a26b26200974"
archive="/Users/yifanxu/Ephemeral-AI-Lab/torch-layerfs-init-20260908/implementation-evidence/venv.tar"
source="/Users/yifanxu/Ephemeral-AI-Lab/torch-uv/.venv"
rows=[]
def run(argv,log,timeout,env=None):
 with log.open("x") as f:
  p=subprocess.Popen(argv,stdout=f,stderr=subprocess.STDOUT,env=env,start_new_session=True)
  try:return p.wait(timeout=timeout)
  except subprocess.TimeoutExpired:
   os.killpg(p.pid,signal.SIGKILL);p.wait();return 124
with open(os.environ["TMPDIR"]+"/layerfs-infra-measurement.lock","a") as lock:
 fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 out.mkdir()
 binaries={"control":control,"candidate":r/"candidate-02-proof"}
 identity={"candidate_commit":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"control_commit":"1e3dce93547c4186484d75e1f9439a5491e0074a","order":list(binaries),"binary_sha256":{k:hashlib.sha256(p.read_bytes()).hexdigest() for k,p in binaries.items()},"image":image,"runtime_reuse":"Only macOS Store admission changed; daemon/FUSE/content/runtime sources unchanged.","archive_sha256":"fdc1d8f63eba66c44af192929544c72d7d56126f155107e29de1c8c0de0e3314"}
 (out/"identity.json").write_text(json.dumps(identity,indent=2))
 for arm,binary in binaries.items():
  argv=[str(binary),"workspace",archive,str(out/arm),image]
  (out/(arm+"-command.json")).write_text(json.dumps(argv))
  code=run(argv,out/(arm+".log"),480)
  text=(out/(arm+".log")).read_text()
  row={"arm":arm,"exit_code":code}
  row.update({k:float(v) for k,v in re.findall(r"^([a-z_]+)=([0-9.]+)$",text,re.M)})
  rows.append(row);(out/"results.json").write_text(json.dumps(rows,indent=2))
  if code:
   for name in re.findall(r"^container=(layerfs-issue71-.+)$",text,re.M):subprocess.run(["docker","rm","-f",name],capture_output=True)
   raise SystemExit(code)
  assert "cleanup=removed_owned_container" in text
  row["branch"]=re.search(r"^branch=(.+)$",text,re.M)[1]
  db=out/arm/"store.sqlite";row["closed_store_logical_bytes"]=db.stat().st_size;row["closed_store_allocated_bytes"]=db.stat().st_blocks*512
  print(json.dumps(row),flush=True)
 for row in rows:
  arm=row["arm"]
  code=run([str(binaries[arm]),"verify",source,str(out/arm/"store.sqlite")],out/(arm+"-verify.log"),180,dict(os.environ,VERIFY_BRANCH=row["branch"]))
  row["verification_exit_code"]=code
  (out/"results.json").write_text(json.dumps(rows,indent=2))
  assert code==0
  print(arm+" full verification PASS",flush=True)
