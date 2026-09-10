"""Bounded native test execution with exact, disjoint process batches."""
import concurrent.futures
import json
import os
import re
import subprocess
import sys
import time


def test_batches(names, jobs):
    count = min(jobs, max(1, (len(names) + 31) // 32))
    return [names[index::count] for index in range(count)]


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


def main(manifest, jobs):
    if not 1 <= jobs <= 16:
        raise SystemExit("test-fast: jobs must be in 1..16")
    executables = {}
    with open(manifest, encoding="utf-8") as source:
        for line in source:
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            executable = message.get("executable")
            manifest_path = message.get("manifest_path")
            if message.get("reason") == "compiler-artifact" and executable and manifest_path and message.get("profile", {}).get("test"):
                executables[executable] = os.path.dirname(manifest_path)
    if not executables:
        raise SystemExit("test-fast: Cargo produced no test executables")

    discovered = []
    for executable, directory in sorted(executables.items()):
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
    # Start large suites early; exact batches keep global test state process-local.
    runnable = [(executable, directory, batch)
                for executable, directory, names in sorted(discovered, key=lambda item: -len(item[2]))
                for batch in test_batches(names, jobs)]
    print(f"test-fast: {sum(len(names) for _, _, names in discovered)} tests/benchmarks, "
          f"{len(executables)} binaries, {len(runnable)} disjoint batches", flush=True)
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
