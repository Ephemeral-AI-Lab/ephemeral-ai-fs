import hashlib,json,pathlib,subprocess,sys,uuid
r=pathlib.Path(__file__).parent;root=r.parent/'layerfs-issue95';campaign=r/'terminal-benchmark';archive=r/('prework-refusal-repair-'+sys.argv[1]);archive.mkdir()
ledger=campaign/'verification-ledger.json';rows=json.loads(ledger.read_text());failed=[x for x in rows if x['status'] not in ['PASS','NOT_RUN_OPTIONAL']]
assert failed,'no failure warrants retry'
for x in failed:
 p=pathlib.Path(x['receipt']);s=json.loads(p.read_text())
 assert s['error']=='RuntimeError: another benchmark owns the measurement lock'
 assert s['checks']==[] and all(s.get(k) is None for k in ['host_executor','product_identity','environment_observation','command_wall_ns','preparation','phase'])
 assert {f.name for f in p.parent.iterdir()} <= {'verification.json','failure.log'}
for name in ['verification-ledger.json','terminal-assessment.json']:
 p=campaign/name
 if p.exists():(archive/name).write_bytes(p.read_bytes())
moves=[]
for x in failed:
 p=pathlib.Path(x['receipt']);digest=hashlib.sha256(p.read_bytes()).hexdigest();dest=p.parent.with_name(p.parent.name+'.prework-refused-'+uuid.uuid4().hex[:8]);p.parent.rename(dest)
 assert hashlib.sha256((dest/p.name).read_bytes()).hexdigest()==digest
 moves.append(dict(original=str(p),archived=str(dest/p.name),sha256=digest,reason='proven pre-lock acquisition refusal; supplied source/input args are fallback metadata, no runtime or Store was started'))
(archive/'custody.json').write_text(json.dumps(moves,indent=2))
cmd=json.loads((r/'terminal-benchmark-command.json').read_text())['argv']+['--phase','verification'];(archive/'command.json').write_text(json.dumps(cmd,indent=2))
with (archive/'output.log').open('w') as f:p=subprocess.run(cmd,cwd=root,stdout=f,stderr=subprocess.STDOUT)
(archive/'result.json').write_text(json.dumps(dict(returncode=p.returncode,scope='Only refused proof executions retried. Existing matching PASS receipts reused by unchanged collector. All198 performance rows reused.')))
print('proof repair exit',p.returncode);sys.exit(p.returncode)
