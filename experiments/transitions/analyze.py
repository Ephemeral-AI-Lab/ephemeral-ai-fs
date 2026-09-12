"""Summarize retained exploratory receipts without rerunning the product."""
import json
from pathlib import Path
import statistics
import sys

performance, verification, destination = map(Path, sys.argv[1:])
perf = json.loads((performance / "summary.json").read_text())
proof = json.loads((verification / "summary.json").read_text())
assert perf["status"] == proof["status"] == "PASS"
grouped = {}
for sample in perf["results"]:
    assert sample["mode"] == "performance" and sample["cleanup_status"] == "PASS"
    steps = [r for r in sample["records"] if r["kind"] == "transition-step"]
    grouped.setdefault(sample["case"], []).append({"repetition": sample["repetition"], "steps": steps,
        "edit_ns": sum(r["edit_ns"] for r in steps), "commit_ns": sum(r["commit_ns"] for r in steps),
        "edit_commit_ns": sum(r["edit_commit_ns"] for r in steps), "complete_wall_ns": sample["complete_wall_ns"]})
    for step in steps:
        assert step["representation"] == ("small" if step["after_bytes"] < 131072 else "chunked")
        assert step["diagnostic_records"] == 1, "CDC observation scope must be unambiguous"
        if sample["case"] in {"edit-128k", "edit-129k", "edit-16m"}:
            assert step["cdc_bytes_scanned"] == 1
        if sample["case"].startswith("shrink-") or sample["case"] in {"edit-64k", "edit-127k", "chain-64k", "batched-crossings"}:
            assert step["cdc_bytes_scanned"] == 0
        if sample["case"] == "oscillate":
            if step["step"] % 2 == 0:
                assert step["cdc_bytes_scanned"] == 129 * 1024
            else:
                assert step["reused_prior_content_root"]
verified = sum(next(r["retained_versions_verified"] for r in s["records"] if r["kind"] == "transition-complete") for s in proof["results"])
summary = {"status": "PASS", "admission_eligible": False, "performance_samples": len(perf["results"]),
    "verification_cases": len(proof["results"]), "retained_versions_verified": verified, "cases": {}}
lines = ["# Hybrid transition experiment results", "", "Exploratory observations on unchanged product code; no optimality or release claim.", "",
    "| Case | Samples | Commits/sample | Median edit ms | Median Commit ms | Median edit+Commit ms | Edit+Commit range ms |", "|---|---:|---:|---:|---:|---:|---:|"]
for case, samples in grouped.items():
    metrics = {key: {"median_ns": statistics.median(s[key] for s in samples),
                     "min_ns": min(s[key] for s in samples), "max_ns": max(s[key] for s in samples)}
               for key in ("edit_ns", "commit_ns", "edit_commit_ns", "complete_wall_ns")}
    summary["cases"][case] = {"samples": len(samples), "commits_per_sample": len(samples[0]["steps"]), "metrics": metrics, "runs": samples}
    ms = lambda key: metrics[key]["median_ns"] / 1e6
    span = metrics["edit_commit_ns"]
    lines.append(f"| {case} | {len(samples)} | {len(samples[0]['steps'])} | {ms('edit_ns'):.3f} | {ms('commit_ns'):.3f} | {ms('edit_commit_ns'):.3f} | {span['min_ns']/1e6:.3f}–{span['max_ns']/1e6:.3f} |")
lines += ["", "Multi-Commit rows are sums per sample, not per-Commit latency.", "",
          f"Correctness: {len(proof['results'])} independent case replays; {verified} saved versions checked byte-for-byte after reopen, including via new FUSE workspaces.", "",
          "Timing scope: public SDK edit and Commit acknowledgements. Setup, FUSE verification, representation inspection, reopen and cleanup are excluded. Raw receipts include complete command/lifecycle walls and host/container resources. Fresh Store/process/container for each sample; OS cache uncontrolled. Representation inspection between history steps may warm subsequent reads. Host lifetime RSS includes fixture/oracle data.", "",
          "CDC input is not total hashing or I/O. Commit decode/encoding counters include metadata. Separate correctness replay timings do not enter performance summaries.", "",
          f"Performance command wall total: {perf['complete_wall_ns']/1e9:.3f} s. Verification command wall total: {proof['complete_wall_ns']/1e9:.3f} s. Build/setup timings are retained separately."]
destination.mkdir(exist_ok=True)
with (destination / "summary.json").open("x") as f:
    json.dump(summary, f, indent=2)
(destination / "results.md").write_text("\n".join(lines) + "\n")
print("\n".join(lines))
