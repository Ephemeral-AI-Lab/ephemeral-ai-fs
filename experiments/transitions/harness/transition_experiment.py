"""Quick exploratory transitions using the existing benchmark runtime and custody checks."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import platform
import shutil
import time
import uuid

import integrated_storage
import runner
import runtime

CASES = ["edit-64k", "edit-127k", "edit-128k", "edit-129k", "edit-16m",
         "oscillate", "batched-crossings", "shrink-1m", "shrink-16m", "chain-64k"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cases", nargs="+", choices=CASES, default=CASES)
    parser.add_argument("--mode", choices=["performance", "verify"], required=True)
    parser.add_argument("--repetitions", type=int, default=1)
    args = parser.parse_args()
    if not 1 <= args.repetitions <= 3:
        parser.error("repetitions must be 1..3")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    started = time.monotonic_ns()
    binary = runner.REPO / "target/release/fs-benchmark-pro"
    with (Path(os.environ.get("TMPDIR", "/tmp")) / "layerfs-infra-measurement.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        source, host_identity, image = integrated_storage.check_identity(binary, args.image)
        integrated_storage.save(output / "identity.json", {
            "schema": "hybrid-transition-exploration-v1", "admission_eligible": False,
            "source": source, "host": host_identity, "image_id": image["Id"],
            "platform": platform.platform(), "cases": args.cases, "mode": args.mode,
            "repetitions": args.repetitions, "cache": "fresh Store/process/container; OS cache uncontrolled; no target pre-read in performance; post-Commit representation inspection can warm later steps",
            "mutation": "Client::edit_workspace_file_range", "projection": "real Docker FUSE",
            "timers": "edit/Commit public calls only; lifecycle includes mount check and end; setup/proof/inspection excluded from call timers",
            "retention": "all raw results retained; successful temporary inputs/Stores removed after checks; failures retained",
        })
        results = []
        for repetition in range(1, args.repetitions + 1):
            # Reverse the next pass to reduce a consistent case-order bias.
            cases = args.cases if repetition % 2 else list(reversed(args.cases))
            for case in cases:
                folder = output / f"r{repetition}-{case}"
                folder.mkdir()
                host = folder / "host-runtime"
                host.mkdir()
                tmp = host / "tmp"
                tmp.mkdir()
                row = {"case": case, "repetition": repetition, "mode": args.mode,
                       "status": "INCOMPLETE", "admission_eligible": False}
                sample = None
                call_started = time.monotonic_ns()
                try:
                    sample = runtime.start_sample(image["Id"], "layerfs-transition-" + uuid.uuid4().hex[:12],
                        {"family": "hybrid-transition-exploration-v1", "run": folder.name}, deadline=runtime.Deadline.after(60))
                    row["container"] = {"id": sample.id, "observation": sample.observation}
                    command = [str(binary), "transition-experiment", str(host), sample.id, case, args.mode]
                    row["command"] = command
                    env = {**os.environ, "TMPDIR": str(tmp), "SQLITE_TMPDIR": str(tmp),
                           "LAYERFS_EXEC_TRANSPORT": "daemon", "LAYERFS_FUSE_TRANSPORT": "daemon"}
                    env.pop("LAYERFS_EDIT_DIAGNOSTIC_NONCE", None)
                    raw = runtime.run(command, deadline=runtime.Deadline.after(120), env=env,
                                      check=False, output_limit=8 * 1024**2)
                    (folder / "stdout.jsonl").write_bytes(raw.stdout)
                    (folder / "stderr.log").write_bytes(raw.stderr)
                    row.update(exit_code=raw.returncode, timed_out=raw.timed_out, command_wall_ns=raw.wall_ns)
                    rows = integrated_storage.records(raw.stdout)
                    row["records"] = rows
                    if raw.returncode or raw.timed_out or raw.truncated or not any(
                        r.get("kind") == "transition-complete" and r.get("status") == "PASS" for r in rows
                    ):
                        raise RuntimeError("transition command failed; see retained raw output")
                    row["status"] = "PASS"
                except Exception as error:
                    row["error"] = repr(error)
                finally:
                    try:
                        if sample:
                            logs = runtime.run(["docker", "logs", sample.id], deadline=runtime.Deadline.after(20), check=False)
                            (folder / "container.log").write_bytes(logs.stdout + logs.stderr)
                            sample.remove(runtime.Deadline.after(30))
                        if row["status"] == "PASS":
                            shutil.rmtree(host)
                        row["cleanup_status"] = "PASS"
                    except Exception as error:
                        row.update(status="INCOMPLETE", cleanup_status="FAIL", cleanup_error=repr(error))
                    row["complete_wall_ns"] = time.monotonic_ns() - call_started
                    integrated_storage.save(folder / "result.json", row)
                results.append(row)
                print(json.dumps({k: row[k] for k in ("case", "repetition", "mode", "status", "complete_wall_ns")}), flush=True)
                if row["status"] != "PASS":
                    break
            if any(row["status"] != "PASS" for row in results):
                break
        status = "PASS" if len(results) == len(args.cases) * args.repetitions and all(r["status"] == "PASS" for r in results) else "INCOMPLETE"
        integrated_storage.save(output / "summary.json", {"status": status, "results": results,
            "complete_wall_ns": time.monotonic_ns() - started, "admission_eligible": False})
        integrated_storage.save(output / "manifest.json", {
            str(path.relative_to(output)): runtime.file_sha256(path)
            for path in sorted(output.rglob("*")) if path.is_file() and "host-runtime" not in path.parts
        })
        return 0 if status == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
