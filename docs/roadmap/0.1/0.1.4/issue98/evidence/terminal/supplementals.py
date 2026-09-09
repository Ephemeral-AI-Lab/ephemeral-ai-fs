import json,pathlib,subprocess,sys,time
r=pathlib.Path(__file__).parent;root=r.parent/'layerfs-workspace-admission';old=root.parent/'layerfs-issue91-runs'
commands=json.loads((old/'r26-preparation/full-benchmark-commands.json').read_text())['commands'];image=(r/'final-image-id.txt').read_text().strip();receipts=[]
for c in commands:
 if c['id'] not in ['small-files-performance','small-files-verification','frequent-edits-performance','frequent-edits-verification']:continue
 argv=[x.replace('sha256:8e74a58284a6e8c22ac9cbf60231f69217c3cae31c231ef17b0afd81726b8249',image).replace(str(old/'r26-build-host-02/fs-benchmark-pro'),str(root/'target/release/fs-benchmark-pro')).replace(str(old/'r26-small-files-final'),str(r/'terminal-small-files')).replace(str(old/'r26-frequent-edits-final'),str(r/'terminal-frequent-edits')) for x in c['argv']]
 out=r/('supplemental-'+c['id']);out.mkdir();(out/'command.json').write_text(json.dumps(dict(argv=argv,cwd=str(root)),indent=2));start=time.monotonic_ns()
 with (out/'output.log').open('w') as f:p=subprocess.run(argv,cwd=root,stdout=f,stderr=subprocess.STDOUT)
 receipt=dict(id=c['id'],returncode=p.returncode,wall_ns=time.monotonic_ns()-start);(out/'result.json').write_text(json.dumps(receipt));receipts.append(receipt);print(receipt,flush=True)
 if p.returncode:sys.exit(p.returncode)
(r/'supplemental-qualification.json').write_text(json.dumps(dict(status='PASS',receipts=receipts),indent=2))
