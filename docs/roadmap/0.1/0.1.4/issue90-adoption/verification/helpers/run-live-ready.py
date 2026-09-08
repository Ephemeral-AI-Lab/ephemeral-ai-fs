import pathlib,subprocess,sys,time
start=time.monotonic()
for attempt in range(90):
    ready=subprocess.run(['docker','info','--format','{{.ServerVersion}} {{.OSType}} {{.Architecture}}'],capture_output=True,text=True,timeout=5)
    if ready.returncode==0:
        print('Docker ready',attempt+1,ready.stdout.strip(),flush=True)
        break
    time.sleep(.5)
else:raise SystemExit('Docker readiness failed')
print('readiness_elapsed_s',time.monotonic()-start,flush=True)
sys.exit(subprocess.run([sys.executable,str(pathlib.Path(__file__).with_name('run-live.py'))],timeout=200).returncode)
