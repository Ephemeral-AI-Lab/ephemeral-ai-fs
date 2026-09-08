import fcntl, json, os, pathlib, subprocess, sys, time
root=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue90-storage-adoption')
out=pathlib.Path(__file__).parent/sys.argv[1]
out.mkdir()
cmd=sys.argv[2:]
(out/'command.json').write_text(json.dumps(cmd))
with (pathlib.Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a+') as lock:
    try: fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
    except BlockingIOError:
        (out/'result.json').write_text(json.dumps({'status':'lock-busy'})); raise
    (out/'source.patch').write_bytes(subprocess.check_output(['git','diff','HEAD','--binary'],cwd=root))
    (out/'source.txt').write_bytes(subprocess.check_output(['git','rev-parse','HEAD','HEAD^{tree}'],cwd=root)+subprocess.check_output(['git','status','--porcelain=v1'],cwd=root))
    start=time.monotonic()
    env=dict(os.environ,CARGO_TARGET_DIR=str(root/'target'),CARGO_BUILD_JOBS='4')
    with (out/'output.log').open('w') as log:
        p=subprocess.run(cmd,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT)
    (out/'result.json').write_text(json.dumps({'exit_code':p.returncode,'elapsed_s':time.monotonic()-start}))
    print((out/'output.log').read_text()[-18000:])
    print((out/'result.json').read_text())
    sys.exit(p.returncode)
