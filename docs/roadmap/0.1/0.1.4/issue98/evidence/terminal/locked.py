import fcntl,json,os,pathlib,subprocess,sys,time
out=pathlib.Path(sys.argv[1]);out.mkdir()
cmd=sys.argv[2:];(out/'command.json').write_text(json.dumps(cmd,indent=2))
with (pathlib.Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
 fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 start=time.monotonic_ns()
 with (out/'output.log').open('w') as f:r=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT)
 (out/'result.json').write_text(json.dumps(dict(returncode=r.returncode,wall_ns=time.monotonic_ns()-start)))
 print('exit',r.returncode,'log',out/'output.log')
 sys.exit(r.returncode)
