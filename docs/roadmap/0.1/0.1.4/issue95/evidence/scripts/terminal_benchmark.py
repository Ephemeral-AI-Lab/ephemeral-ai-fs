import json,pathlib,subprocess,sys,time
root=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue95');runs=pathlib.Path(__file__).parent
image=(runs/'c2-image-id.txt').read_text().strip()
cmd=['python3','benchmark/fs-bench-pro/issue54_collect.py','--checkpoint','--campaign','docs/roadmap/0.1/0.1.4/issue91-campaign/declaration.json','--proof-preparation-declaration','docs/roadmap/0.1/0.1.4/issue91-campaign/verification-preparation-r4.json','--image',image,'--host-binary',str(root/'target/release/fs-benchmark-pro'),'--output',str(runs/'terminal-benchmark')]
(runs/'terminal-benchmark-command.json').write_text(json.dumps(dict(argv=cmd,cwd=str(root)),indent=2))
start=time.monotonic_ns()
with (runs/'terminal-benchmark.log').open('x') as f:r=subprocess.run(cmd,cwd=root,stdout=f,stderr=subprocess.STDOUT)
(runs/'terminal-benchmark-result.json').write_text(json.dumps(dict(returncode=r.returncode,wall_ns=time.monotonic_ns()-start)))
print('benchmark exit',r.returncode,flush=True);sys.exit(r.returncode)
