#!/usr/bin/env python3
"""Single-use bounded selection supervisor; no Store implementation."""
import fcntl,json,os,pathlib,resource,shutil,signal,subprocess,sys,time

def main():
 out=pathlib.Path(sys.argv[1]);command=sys.argv[2:];out.mkdir()
 with (pathlib.Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
  fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
  start=time.monotonic_ns();limit=1800*10**9;peak=0;free_min=shutil.disk_usage(out).free;samples=0;failure=None
  with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
   proc=subprocess.Popen(command,stdout=stdout,stderr=stderr,start_new_session=True)
   try:
    while proc.poll() is None:
     table=subprocess.check_output(['ps','-axo','pid=,ppid=,rss='],text=True,timeout=10)
     rows=[tuple(map(int,line.split())) for line in table.splitlines()];owners={proc.pid}
     for _ in range(8):
      children={pid for pid,ppid,rss in rows if ppid in owners}
      if children<=owners:break
      owners|=children
     current=sum(rss*1024 for pid,ppid,rss in rows if pid in owners);peak=max(peak,current);samples+=1
     free_min=min(free_min,shutil.disk_usage(out).free)
     if current>8*1024**3:failure='sampled process-tree RSS >8GiB'
     if free_min<50*1024**3:failure='free disk <50GiB'
     if time.monotonic_ns()-start>limit:failure='selection supervisor >1800s'
     if failure:os.killpg(proc.pid,signal.SIGTERM);break
     time.sleep(0.25)
    try:code=proc.wait(timeout=10)
    except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);code=proc.wait()
   finally:
    if proc.poll() is None:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
  lifetime=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
  if lifetime>8*1024**3:failure='child process-lifetime maximum RSS >8GiB'
  value={'status':'PASS' if code==0 and failure is None else 'FAIL','exit_code':code,'failure':failure,'command':command,'elapsed_ns':time.monotonic_ns()-start,'sampled_process_tree_rss_max_bytes':peak,'child_lifetime_rss_max_bytes':lifetime,'rss_scope':'macOS ps process-tree sum at250ms plus observer time; lifetime rusage child high-water distinct, not exact phase peak','samples_count':samples,'minimum_sampled_free_bytes':free_min,'supervisor_cpu_observer_scope':'preparation only, not read measurement'}
  (out/'completion.json').write_text(json.dumps(value,indent=2)+'\n');print(json.dumps(value));raise SystemExit(0 if value['status']=='PASS' else 1)
if __name__=='__main__':main()
