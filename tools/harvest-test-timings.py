"""Regenerate the execution-order hint consumed by tools/test_fast.py.

The suite itself never depends on this table: every test still runs exactly once and
every batch still has to report its exact completion count. The table only decides how
tests are grouped into process batches and in which order those batches are submitted.

Usage:

    cargo test --workspace --all-features --locked --no-run --message-format=json >/tmp/artifacts.jsonl
    python3 tools/harvest-test-timings.py --manifest /tmp/artifacts.jsonl

Per-test durations come from the libtest ``--report-time`` flag, which is accepted on a
stable toolchain only when ``RUSTC_BOOTSTRAP=1`` and ``-Z unstable-options`` are set; the
harvest sets both for the child process it launches.
"""
import argparse
import concurrent.futures
import json
import os
import re
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import test_fast  # noqa: E402

REPORT = re.compile(r"^test (\S+) \.\.\. \w+(?: <([0-9.]+)s>)?$", re.M)


def measure(item):
    executable, directory, names = item
    result = subprocess.run(
        [executable, "-Z", "unstable-options", "--test-threads=1", "--exact", "--report-time", *names],
        cwd=directory, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        env=dict(os.environ, RUSTC_BOOTSTRAP="1"),
    )
    if result.returncode != 0:
        raise SystemExit(f"harvest: {os.path.basename(executable)} failed:\n{result.stdout[-4000:]}")
    measured = {name: float(seconds) for name, seconds in REPORT.findall(result.stdout) if seconds}
    unknown = sorted(set(measured) - set(names))
    if unknown:
        raise SystemExit(f"harvest: {os.path.basename(executable)} reported unknown tests: {unknown[:3]}")
    return measured


COMMENT = ("Execution-order hint for tools/test_fast.py: observed seconds per test on a developer host. "
           "Only the relative cost matters, every test still runs, and a missing entry only costs ordering. "
           "Regenerate with: python3 tools/harvest-test-timings.py --manifest <cargo json>")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, help="cargo --message-format=json build output")
    parser.add_argument("--jobs", type=int, default=4, help="binaries measured at once (default 4)")
    parser.add_argument("--output", default=test_fast.TIMINGS)
    arguments = parser.parse_args()

    discovered = test_fast.discover(test_fast.executables(arguments.manifest))
    runnable = [(executable, directory, names) for executable, directory, names in discovered if names]
    started = time.monotonic()
    measured = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=arguments.jobs) as pool:
        for result in pool.map(measure, runnable):
            measured.update(result)
    try:
        with open(arguments.output, encoding="utf-8") as source:
            payload = json.load(source)
    except (OSError, ValueError):
        payload = {}
    payload["comment"] = payload.get("comment", COMMENT)
    payload["tests"] = {name: rounded for name, rounded in
                        sorted((name, round(seconds, 2)) for name, seconds in measured.items() if seconds > 0)}
    with open(arguments.output, "w", encoding="utf-8") as sink:
        json.dump(payload, sink, indent=1, sort_keys=True)
        sink.write("\n")
    print(f"harvest: {len(payload['tests'])} durations from {len(runnable)} binaries "
          f"in {time.monotonic() - started:.1f}s -> {arguments.output}")


if __name__ == "__main__":
    main()
