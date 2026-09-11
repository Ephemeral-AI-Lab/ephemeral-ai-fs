"""Count docker CLI invocations per sample by wrapping runtime.run. Read-only."""
import fcntl, json, os, sys, time, uuid
from pathlib import Path
ARM = Path("/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/C")
sys.path.insert(0, str(ARM / "benchmark/fs-bench-pro/shared"))
import runner, runtime

CALLS = []
orig = runtime.run
def counting(argv, **kw):
    if argv and str(argv[0]) == "docker":
        t0 = time.monotonic_ns()
        try:
            return orig(argv, **kw)
        finally:
            CALLS.append((argv[1] if len(argv) > 1 else "?", time.monotonic_ns() - t0))
    return orig(argv, **kw)
runtime.run = counting

lock = Path(os.environ.get("TMPDIR","/tmp"))/"layerfs-infra-measurement.lock"
image = Path("/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/evidence/C-image.txt").read_text().strip()
parser = runner.build_parser()
args = parser.parse_args(["--family","dedup_cdc_locality","--case","dedup-cdc-scattered-100",
                          "--seed","1","--setup","fresh","--image",image,
                          "--product-timeout","300","--timeout","310","--setup-timeout","600"])
args.output = "/tmp/issue114-count-run"
with lock.open("a") as lk:
    fcntl.flock(lk, fcntl.LOCK_EX)
    ALL=[]
    W=[]
    for i in range(5):
        CALLS.clear()
        r = runner.execute_selected(args, deadline=time.monotonic()+900)
        ALL.append((list(CALLS), r["wall_ns"]/1e6, r["preparation_wall_ns"]/1e6, r["cleanup"]["wall_ns"]/1e6))
import statistics as st
print("%-4s %6s %10s %10s %10s %10s"%("i","ncalls","docker_ms","wall_ms","prep_ms","clean_ms"))
for i,(c,w,p,cl) in enumerate(ALL,1):
    print("%-4d %6d %10.1f %10.1f %10.1f %10.1f"%(i,len(c),sum(d for _,d in c)/1e6,w,p,cl))
print()
print("median calls", st.median([len(c) for c,_,_,_ in ALL]))
print("median docker ms %.1f  wall %.1f  -> docker share %.1f%%"%(
  st.median([sum(d for _,d in c)/1e6 for c,_,_,_ in ALL]),
  st.median([w for _,w,_,_ in ALL]),
  100*st.median([sum(d for _,d in c)/1e6 for c,_,_,_ in ALL])/st.median([w for _,w,_,_ in ALL])))
