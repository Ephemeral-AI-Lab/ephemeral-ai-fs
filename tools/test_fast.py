"""Bounded native test execution with exact, disjoint process batches.

Every test and benchmark still runs exactly once, and every batch still has to report
its exact completion count. Batches are sized and submitted with a committed per-test
timing hint (``tools/test-fast-timings.json``) so the bounded worker slots stay busy:
the hint only orders execution, it never selects, skips, or weakens a test.
"""
import concurrent.futures
import json
import math
import os
import re
import subprocess
import sys
import time

DEFAULT_SECONDS = 0.3
PROCESS_SECONDS = 0.2
MAX_BATCH_TESTS = 12
TARGET_BATCH_SECONDS = 10.0
TIMINGS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "test-fast-timings.json")


def load_timings(path=TIMINGS):
    """Read the execution-order hint; a missing or damaged file only costs ordering."""
    try:
        with open(path, encoding="utf-8") as source:
            payload = json.load(source)
    except (OSError, ValueError):
        return {}
    entries = payload.get("tests")
    if not isinstance(entries, dict):
        return {}
    return {name: float(seconds) for name, seconds in entries.items()
            if isinstance(seconds, (int, float)) and not isinstance(seconds, bool) and seconds >= 0}


def seconds_for(name, timings):
    return timings.get(name, DEFAULT_SECONDS)


def split_binary(names, timings):
    """Balance one binary's tests over the batches its test count and cost ask for."""
    if not names:
        return []
    count = max(1, min(len(names),
                       max(-(-len(names) // MAX_BATCH_TESTS),
                           math.ceil(sum(seconds_for(name, timings) for name in names) / TARGET_BATCH_SECONDS))))
    buckets = [[] for _ in range(count)]
    loads = [0.0] * count
    for name in sorted(names, key=lambda name: (-seconds_for(name, timings), name)):
        index = min((candidate for candidate in range(count) if len(buckets[candidate]) < MAX_BATCH_TESTS),
                    key=lambda candidate: (loads[candidate], candidate))
        buckets[index].append(name)
        loads[index] += seconds_for(name, timings)
    return [sorted(bucket) for bucket in buckets if bucket]


def plan_batches(discovered, timings):
    """Longest-first list scheduling: each freeing worker picks up the heaviest batch left."""
    plan = [(PROCESS_SECONDS + sum(seconds_for(name, timings) for name in batch), executable, directory, batch)
            for executable, directory, names in discovered
            for batch in split_binary(names, timings)]
    plan.sort(key=lambda item: (-item[0], -len(item[3]), item[1], item[3][0]))
    return [(executable, directory, batch) for _, executable, directory, batch in plan]


def run(item):
    executable, working_directory, names = item
    started = time.monotonic()
    result = subprocess.run(
        [executable, "--test-threads=1", "--exact", *names],
        cwd=working_directory,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    counts = re.findall(r"test result: .*? (\d+) passed; (\d+) failed; (\d+) ignored;", result.stdout)
    status = result.returncode
    if not counts or sum(map(int, counts[-1])) != len(names):
        status = 1
        result.stdout += "\ntest-fast: selected test count differs from completion summary\n"
    return executable, status, time.monotonic() - started, result.stdout


def executables(manifest_path):
    executables = {}
    with open(manifest_path, encoding="utf-8") as source:
        for line in source:
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            executable = message.get("executable")
            manifest = message.get("manifest_path")
            if message.get("reason") == "compiler-artifact" and executable and manifest and message.get("profile", {}).get("test"):
                executables[executable] = os.path.dirname(manifest)
    return executables


def discover(known):
    discovered = []
    for executable, directory in sorted(known.items()):
        listing = subprocess.check_output(
            [executable, "--list", "--format=terse"], cwd=directory, text=True
        )
        names = []
        for line in listing.splitlines():
            if not line.endswith((": test", ": benchmark")):
                raise SystemExit(f"test-fast: unrecognized test listing: {line}")
            names.append(line.rsplit(": ", 1)[0])
        if len(names) != len(set(names)):
            raise SystemExit("test-fast: duplicate test names")
        discovered.append((executable, directory, names))
    return discovered


def main(manifest, jobs):
    if not 1 <= jobs <= 16:
        raise SystemExit("test-fast: jobs must be in 1..16")
    known = executables(manifest)
    if not known:
        raise SystemExit("test-fast: Cargo produced no test executables")

    discovered = discover(known)
    runnable = plan_batches(discovered, load_timings())
    print(f"test-fast: {sum(len(names) for _, _, names in discovered)} tests/benchmarks, "
          f"{len(known)} binaries, {len(runnable)} disjoint batches, longest-first plan", flush=True)
    failed = False
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        futures = [pool.submit(run, item) for item in runnable]
        for future in concurrent.futures.as_completed(futures):
            executable, status, elapsed, output = future.result()
            print(f"=== {os.path.basename(executable)} ({elapsed:.2f}s, status={status}) ===", flush=True)
            print(output, end="" if output.endswith("\n") else "\n", flush=True)
            failed |= status != 0
    if failed:
        raise SystemExit("test-fast: one or more native test batches failed")


if __name__ == "__main__":
    main(sys.argv[1], int(sys.argv[2]))
