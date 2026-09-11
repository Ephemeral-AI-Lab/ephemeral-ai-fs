#!/usr/bin/env python3
"""issue #114 STEP 1 probe: attribute preparation_wall_ns / cleanup.wall_ns.

Mechanism: import the *unmodified* frozen runner and runtime modules from the
arm C worktree, wrap the harness entry points that runner.execute_selected calls
between its own timing marks, then let the untouched runner run real samples.
The wrappers only record enter/exit monotonic timestamps; they change no
behaviour, no fixture, no timer, no threshold and no verifier.  The product
image is the frozen arm C image tag.

Emitted per sample:
  host_acquire_ns      runner._host_acquire
  start_sample_ns      runtime.start_sample  (docker create/start/readiness)
  host_sample_ns       runner._host_sample
  cgroup_before/after  runner.cgroup_snapshot
  remove_ns            runtime.SampleContainer.remove (docker rm --force)
  remove_host_owned_ns runtime.remove_host_owned
plus the runner's own preparation_wall_ns / command_wall_ns / cleanup.wall_ns /
wall_ns / pure_call_sum_ns, so the decomposition is self-checking.
"""
import fcntl
import json
import os
import sys
import time
from pathlib import Path

ARM = Path(sys.argv[1] if len(sys.argv) > 1 else
           "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/C")
OUT = Path(sys.argv[2])
ITERATIONS = int(sys.argv[3]) if len(sys.argv) > 3 else 11

sys.path.insert(0, str(ARM / "benchmark/fs-bench-pro/shared"))

import runner  # noqa: E402
import runtime  # noqa: E402

LOG = []


def wrap(module, attr, key):
    original = getattr(module, attr)

    def instrumented(*a, **kw):
        t0 = time.monotonic_ns()
        try:
            return original(*a, **kw)
        finally:
            LOG.append({"key": key, "enter": t0, "exit": time.monotonic_ns()})

    setattr(module, attr, instrumented)


def main():
    OUT.parent.mkdir(parents=True, exist_ok=True)
    lock_path = Path(os.environ.get("TMPDIR", "/tmp")) / "layerfs-infra-measurement.lock"
    image = Path("/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/evidence/C-image.txt").read_text().strip()

    parser = runner.build_parser()
    args = parser.parse_args([
        "--family", "dedup_cdc_locality", "--case", "dedup-cdc-scattered-100",
        "--seed", "1", "--setup", "fresh", "--image", image,
        "--product-timeout", "300", "--timeout", "310", "--setup-timeout", "600",
    ])

    runner_dir = OUT.parent / (OUT.stem + "-runner")
    args.output = str(runner_dir)

    with lock_path.open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        deadline = time.monotonic() + args.timeout + args.setup_timeout + 120

        wrap(runner, "_host_acquire", "host_acquire_ns")
        wrap(runtime, "start_sample", "start_sample_ns")
        wrap(runner, "_host_sample", "host_sample_ns")
        wrap(runner, "cgroup_snapshot", "cgroup_snapshot_ns")
        wrap(runtime.SampleContainer, "remove", "remove_ns")
        wrap(runtime, "remove_host_owned", "remove_host_owned_ns")

        rows = []
        with OUT.open("w") as stream:
            for index in range(1, ITERATIONS + 1):
                LOG.clear()
                result = runner.execute_selected(args, deadline=deadline)
                sc = [r for r in result.get("records", []) if r.get("kind") == "sample-complete"]
                row = {
                    "kind": "probe-sample",
                    "iteration": index,
                    "status": result.get("status"),
                    "runner_wall_ns": result.get("wall_ns"),
                    "runner_prep_wall_ns": result.get("preparation_wall_ns"),
                    "runner_cmd_wall_ns": result.get("command_wall_ns"),
                    "runner_cleanup_ns": (result.get("cleanup") or {}).get("wall_ns"),
                    "pure_call_sum_ns": sc[0]["pure_call_sum_ns"] if sc else None,
                    "observed_ns": {},
                }
                for entry in LOG:
                    k = entry["key"]
                    row["observed_ns"][k] = row["observed_ns"].get(k, 0) + (entry["exit"] - entry["enter"])
                stream.write(json.dumps(row, sort_keys=True) + "\n")
                stream.flush()
                rows.append(row)
                print(json.dumps({k: v for k, v in row.items() if k != "observed_ns"},
                                 sort_keys=True), flush=True)

    return 0


if __name__ == "__main__":
    sys.exit(main())
