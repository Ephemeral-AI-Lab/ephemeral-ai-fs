#!/usr/bin/env python3
"""issue #114 Direction 2 probe: where does the cgroup_snapshot cost live?

Read-only w.r.t. product/fixtures/timers.  Starts one throwaway container using the
harness's own runtime.start_sample (so the cost model is the real one), then times:

  A. the current implementation: docker exec sh -c 'cat ...'
  B. bare `docker exec <id> true`            -> docker CLI + exec round-trip floor
  C. `docker exec <id> cat <file>`           -> without the sh -c layer
  D. a single exec reading all five files    -> current shape, one process
  E. `docker inspect --format` from the HOST -> no exec at all
  F. reading the cgroup via the container's   -> host-side path (if visible)
     projected /sys path

No product, fixture, timer or threshold is touched.  The container is removed at
the end via the same runtime helper the runner uses.
"""
import fcntl
import json
import os
import statistics as st
import sys
import time
import uuid
from pathlib import Path

ARM = Path("/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/C")
OUT = Path(sys.argv[1])
N = int(sys.argv[2]) if len(sys.argv) > 2 else 15

sys.path.insert(0, str(ARM / "benchmark/fs-bench-pro/shared"))
import runner  # noqa: E402
import runtime  # noqa: E402


def timed(fn, reps=1):
    t0 = time.monotonic_ns()
    for _ in range(reps):
        fn()
    return (time.monotonic_ns() - t0) // reps


def main():
    OUT.parent.mkdir(parents=True, exist_ok=True)
    lock = Path(os.environ.get("TMPDIR", "/tmp")) / "layerfs-infra-measurement.lock"
    image = Path("/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/evidence/C-image.txt").read_text().strip()

    with lock.open("a") as lk:
        fcntl.flock(lk, fcntl.LOCK_EX)
        deadline = time.monotonic() + 300
        name = "layerfs-infra-sample-" + uuid.uuid4().hex[:12]
        sample = runtime.start_sample(image, name, {"family": "probe", "run": name},
                                      deadline=runtime.Deadline.after(120), cpus=2, memory_bytes=2 * 1024**3)
        rows = []
        try:
            for i in range(1, N + 1):
                cid = sample.id
                row = {"iteration": i}

                # A. current shape, via the harness function itself
                row["A_current_cgroup_snapshot_ns"] = timed(
                    lambda: runner.cgroup_snapshot(sample, time.monotonic() + 30))

                # B. docker CLI + exec floor
                row["B_exec_true_ns"] = timed(lambda: runtime.run(
                    ["docker", "exec", cid, "true"], deadline=runtime.Deadline.after(30), output_limit=4096))

                # C. no sh -c wrapper
                row["C_exec_cat_cpu_stat_ns"] = timed(lambda: runtime.run(
                    ["docker", "exec", cid, "cat", "/sys/fs/cgroup/cpu.stat"],
                    deadline=runtime.Deadline.after(30), output_limit=4096))

                # D. one exec, all five files (current shape)
                row["D_exec_sh_cat_all_ns"] = timed(lambda: runtime.run(
                    ["docker", "exec", cid, "sh", "-c",
                     "cat /sys/fs/cgroup/cpu.stat; printf 'memory_peak '; cat /sys/fs/cgroup/memory.peak; "
                     "printf 'memory_current '; cat /sys/fs/cgroup/memory.current; "
                     "printf 'swap_current '; cat /sys/fs/cgroup/memory.swap.current; "
                     "cat /sys/fs/cgroup/memory.events"],
                    deadline=runtime.Deadline.after(30), output_limit=4096))

                # E. host-side docker inspect (no container exec)
                row["E_inspect_ns"] = timed(lambda: runtime.run(
                    ["docker", "inspect", "--format", "{{.State.Pid}}", cid],
                    deadline=runtime.Deadline.after(30), output_limit=4096))

                # F. host-side bare docker CLI floor (no API call at all)
                row["F_docker_version_ns"] = timed(lambda: runtime.run(
                    ["docker", "version", "--format", "{{.Server.Version}}"],
                    deadline=runtime.Deadline.after(30), output_limit=4096))

                # G. is the container's cgroup host-visible (would allow a zero-exec read)?
                pid = runtime.run(["docker", "inspect", "--format", "{{.State.Pid}}", cid],
                                  deadline=runtime.Deadline.after(30), output_limit=4096).stdout_text().strip()
                row["G_container_pid"] = pid
                candidates = [Path(f"/sys/fs/cgroup/system.slice/docker-{cid}.scope"),
                              Path(f"/proc/{pid}/cgroup")]
                row["G_host_cgroup_visible"] = any(c.exists() for c in candidates)

                rows.append(row)
                print(json.dumps({k: v for k, v in row.items() if k != "G_container_pid"}), flush=True)
        finally:
            sample.remove(deadline=runtime.Deadline.after(120))

    with OUT.open("w") as f:
        for r in rows:
            f.write(json.dumps(r, sort_keys=True) + "\n")

    print("\n--- medians (n=%d) ---" % len(rows))
    numeric = [k for k in rows[0] if k not in ("G_container_pid", "G_host_cgroup_visible")]
    for key in numeric:
        v = [r[key] for r in rows]
        print("  %-32s %8.2f ms   (min %7.2f max %7.2f)" % (key, st.median(v) / 1e6, min(v) / 1e6, max(v) / 1e6))
    print("  host cgroup visible: %s" % rows[0]["G_host_cgroup_visible"])


if __name__ == "__main__":
    main()
