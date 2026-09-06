#!/usr/bin/env python3
"""Issue #54 remaining-family statistics collection through the existing runner."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
sys.path.insert(0, str(HERE / "shared"))
import runner

PERF_FAMILIES = (
    "namespace_mutation",
    "directory_construction_traversal",
    "workspace_change_locality",
    "dedup_branch_history",
    "git_tool_workflow",
    "mixed_load_bearing",
)
PROOF_FAMILY = "workspace_reliability"
EXPECTED_PERF = {
    "namespace_mutation": 4,
    "directory_construction_traversal": 12,
    "workspace_change_locality": 16,
    "dedup_branch_history": 20,
    "git_tool_workflow": 4,
    "mixed_load_bearing": 4,
}
EXPECTED_PROOFS = 28
PRODUCT_TIMEOUT = 300
COMMAND_TIMEOUT = 310
SETUP_TIMEOUT = 600
SEED = 1


def _run(argv, cwd=None):
    return subprocess.run(argv, cwd=cwd or REPO, text=True, capture_output=True)


def _sha256(path: Path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _write(path: Path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def list_family(args, family):
    listed = _run([
        sys.executable, str(HERE / "shared/runner.py"),
        "--family", family, "--list", "--image", args.image,
        "--host-binary", args.host_binary,
    ])
    if listed.returncode != 0:
        raise RuntimeError(f"infra-list {family} failed: {listed.stderr[-4000:]}")
    payload = json.loads(listed.stdout)
    return payload["rows"], payload


def perf_status(path: Path):
    if not path.is_file():
        return None
    last = None
    with path.open() as stream:
        for line in stream:
            last = json.loads(line)
    return last


def collect_row(args, family, row, output):
    case = row["scenario_id"]
    proof_only = bool(row.get("proof_only"))
    case_dir = output / ("verification" if proof_only else "performance") / family / case
    receipt_name = "verification.json" if proof_only else "perf.jsonl"
    existing = case_dir / receipt_name
    result = {
        "family": family,
        "case": case,
        "proof_only": proof_only,
        "fixture_profile": row.get("fixture_profile"),
        "tier": row.get("tier"),
        "seed": SEED,
        "sample_count": 0 if proof_only else 1,
        "route": row.get("route"),
    }
    if proof_only:
        return result, "proof"
    if existing.is_file():
        summary = perf_status(existing)
        if summary and summary.get("kind") == "summary" and summary.get("status") in ("PASS", "TARGET_MISS"):
            result.update(reused=True, status=summary["status"], receipt=str(existing),
                          receipt_sha256=_sha256(existing))
            return result, "reuse-pass"
        if summary and summary.get("kind") == "summary":
            result.update(reused=True, status=summary.get("status"), receipt=str(existing),
                          receipt_sha256=_sha256(existing))
            return result, "reuse-incomplete"
    case_dir.mkdir(parents=True, exist_ok=True)
    command = [
        sys.executable, str(HERE / "shared/runner.py"),
        "--family", family, "--case", case, "--seed", str(SEED),
        "--setup", "clone", "--perf-fast", "--collection-mode",
        "--product-timeout", str(PRODUCT_TIMEOUT), "--timeout", str(COMMAND_TIMEOUT),
        "--setup-timeout", str(SETUP_TIMEOUT),
        "--image", args.image, "--host-binary", args.host_binary,
        "--output", str(case_dir),
    ]
    started = time.monotonic()
    proc = _run(command)
    result.update(
        status="PASS" if proc.returncode == 0 else "FAIL",
        returncode=proc.returncode,
        wall_seconds=time.monotonic() - started,
        stdout_tail=proc.stdout[-4000:],
        stderr_tail=proc.stderr[-4000:],
        receipt=str(existing) if existing.is_file() else None,
    )
    if existing.is_file():
        result["receipt_sha256"] = _sha256(existing)
        summary = perf_status(existing)
        if summary:
            sample = None
            header = None
            with existing.open() as stream:
                for line in stream:
                    parsed = json.loads(line)
                    if parsed.get("kind") == "header":
                        header = parsed
                    elif parsed.get("kind") == "sample":
                        sample = parsed
            if sample:
                result["execution_status"] = sample.get("status")
                result["completion_status"] = sample.get("completion_status")
                result["historical_product_target_status"] = sample.get("historical_product_target_status")
                result["cleanup"] = sample.get("cleanup")
                result["resources"] = sample.get("resources")
                result["preparation_wall_ns"] = sample.get("preparation_wall_ns")
                result["command_wall_ns"] = sample.get("command_wall_ns")
                result["error"] = sample.get("error")
                timer, elapsed = runner._timer(sample)
                result["timer"] = timer
                result["elapsed_ns"] = elapsed
                identities = sample.get("identities") or (header or {}).get("identities") or {}
                result["identities"] = {
                    "source_identity": identities.get("source_identity"),
                    "product_identity": identities.get("product_identity"),
                    "harness_identity": identities.get("harness_identity"),
                    "input_identity": identities.get("input_identity"),
                    "image": identities.get("image"),
                    "setup_identity": identities.get("setup_identity"),
                }
            if summary.get("status"):
                result["status"] = summary["status"]
    else:
        result["status"] = "TIMEOUT" if "timeout" in (proc.stderr + proc.stdout).lower() else "FAIL"
    return result, "ran"


def verify_row(args, family, row, output, identities):
    case = row["scenario_id"]
    case_dir = output / "verification" / family / case
    receipt = case_dir / "verification.json"
    result = {"family": family, "case": case, "seed": SEED}
    if case.endswith("sustained-600s-compact-v2-proof") or row.get("kind") == "sustained-600s" or not row.get("verification_supported", True):
        result.update(
            status="INCOMPLETE",
            exception="duration-incompatible",
            omissions=["sustained-600s cannot fit the 60-second verification ceiling or 300-second performance allowance"],
            follow_up="preserve original 600-second definition; do not shorten; track under a dedicated proof issue",
        )
        _write(case_dir / "exception.json", result)
        return result
    if receipt.is_file():
        saved = json.loads(receipt.read_text())
        result.update(status=saved.get("status"), reused=True, receipt=str(receipt),
                      receipt_sha256=_sha256(receipt), wall_seconds=saved.get("wall_seconds"))
        return result
    if not identities or not identities.get("source_identity") or not identities.get("input_identity"):
        result.update(status="INCOMPLETE", error="missing identity-matched performance/source identities")
        return result
    case_dir.mkdir(parents=True, exist_ok=True)
    command = [
        sys.executable, str(HERE / "verify-selected.py"),
        "--family", family, "--case", case, "--seed", str(SEED),
        "--setup", identities.get("setup_identity") or "clone",
        "--source", identities["source_identity"],
        "--input", identities["input_identity"],
        "--image", args.image, "--host-binary", args.host_binary,
        "--output", str(case_dir),
        "--setup-timeout", str(SETUP_TIMEOUT),
    ]
    started = time.monotonic()
    proc = _run(command)
    result.update(returncode=proc.returncode, wall_seconds=time.monotonic() - started,
                  stdout_tail=proc.stdout[-4000:], stderr_tail=proc.stderr[-4000:])
    if receipt.is_file():
        saved = json.loads(receipt.read_text())
        result.update(status=saved.get("status"), receipt=str(receipt), receipt_sha256=_sha256(receipt),
                      error=saved.get("error"), cleanup=saved.get("cleanup"),
                      omissions=saved.get("omissions"), wall_seconds=saved.get("wall_seconds"))
    else:
        result["status"] = "TIMEOUT" if "timeout" in (proc.stderr + proc.stdout).lower() else "INCOMPLETE"
        result["error"] = (proc.stderr or proc.stdout)[-2000:]
    return result


def parse_args():
    parser = argparse.ArgumentParser(description="Collect issue #54 remaining-family statistics")
    parser.add_argument("--image", default=os.environ.get("LAYERFS_BENCH_IMAGE"), required=not os.environ.get("LAYERFS_BENCH_IMAGE"))
    parser.add_argument("--host-binary", default=str(REPO / "target/release/fs-benchmark-pro"))
    parser.add_argument("--output", default=str(REPO / "benchmark-results/host-store/campaigns/issue54"))
    parser.add_argument("--phase", choices=("inventory", "performance", "verification", "all"), default="all")
    parser.add_argument("--family")
    parser.add_argument("--case")
    parser.add_argument("--proofs", choices=("selected", "all"), default="selected",
                       help="selected (default): compact/tier-1/10 plus reliability short proofs; all: every completed case")
    return parser.parse_args()


def main():
    args = parse_args()
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    inventory = {"families": {}, "performance_cases": [], "proof_cases": [], "mismatches": []}
    identities_by_case = {}
    families = PERF_FAMILIES + ((PROOF_FAMILY,) if args.family in (None, PROOF_FAMILY) else ())
    if args.family:
        families = (args.family,)
    for family in families:
        rows, listed = list_family(args, family)
        if args.case:
            rows = [row for row in rows if row["scenario_id"] == args.case]
        inventory["families"][family] = {
            "count": len(rows),
            "source_identity": listed.get("source_identity"),
            "image": listed.get("image"),
            "cases": [row["scenario_id"] for row in rows],
            "proof_only": [row["scenario_id"] for row in rows if row.get("proof_only")],
        }
        expected = EXPECTED_PROOFS if family == PROOF_FAMILY else EXPECTED_PERF.get(family)
        if expected is not None and len(rows) != expected:
            inventory["mismatches"].append({"family": family, "expected": expected, "actual": len(rows)})
        for row in rows:
            if row.get("proof_only"):
                inventory["proof_cases"].append(row["scenario_id"])
            else:
                inventory["performance_cases"].append(row["scenario_id"])
    _write(output / "selections.json", inventory)
    print(json.dumps({
        "phase": "inventory",
        "performance_cases": len(inventory["performance_cases"]),
        "proof_cases": len(inventory["proof_cases"]),
        "mismatches": inventory["mismatches"],
    }), flush=True)
    if args.phase == "inventory":
        return 0 if not inventory["mismatches"] else 1

    ledger = []
    if args.phase in ("performance", "all"):
        for family in families:
            if family == PROOF_FAMILY:
                continue
            rows, _ = list_family(args, family)
            if args.case:
                rows = [row for row in rows if row["scenario_id"] == args.case]
            for row in rows:
                if row.get("proof_only"):
                    continue
                result, how = collect_row(args, family, row, output)
                ledger.append(result)
                _write(output / "performance-ledger.json", ledger)
                print(f"PERF {family} {row['scenario_id']} {result.get('status')} {how} timer={result.get('timer')} elapsed_ns={result.get('elapsed_ns')}", flush=True)
                if result.get("identities"):
                    identities_by_case[row["scenario_id"]] = result["identities"]
        _write(output / "performance-ledger.json", ledger)

    proofs = []
    if args.phase in ("verification", "all"):
        identity_cache = identities_by_case
        if (output / "performance-ledger.json").is_file():
            for item in json.loads((output / "performance-ledger.json").read_text()):
                if item.get("identities"):
                    identity_cache[item["case"]] = item["identities"]
        for family in families:
            rows, listed = list_family(args, family)
            if args.case:
                rows = [row for row in rows if row["scenario_id"] == args.case]
            for row in rows:
                identities = identity_cache.get(row["scenario_id"])
                if row.get("proof_only"):
                    identities = identities or {
                        "source_identity": listed.get("source_identity"),
                        "setup_identity": "clone",
                    }
                    listed_row = row
                    # Resolve input identity through the runner without executing.
                    resolved = _run([
                        sys.executable, str(HERE / "shared/runner.py"),
                        "--family", family, "--case", row["scenario_id"], "--seed", str(SEED),
                        "--list", "--image", args.image, "--host-binary", args.host_binary,
                    ])
                    if resolved.returncode == 0:
                        payload = json.loads(resolved.stdout)
                        # --list with a case still lists; resolve_selection needs execute path.
                    resolve = argparse.Namespace(
                        family=family, case=row["scenario_id"], seed=SEED, image=args.image,
                        host_binary=args.host_binary, topology="host-store",
                        setup="clone", source=None, input=None, cpus=2, memory_mib=2048,
                        timeout=COMMAND_TIMEOUT, product_timeout=PRODUCT_TIMEOUT,
                        setup_timeout=SETUP_TIMEOUT, source_arm="candidate",
                        performance_rows="-", repetition=None, smoke=False, list=False,
                        verification=True, prepare_only=False,
                    )
                    try:
                        selection = runner.resolve_selection(resolve, time.monotonic() + 30)
                        identities = {
                            "source_identity": selection["source_identity"],
                            "input_identity": selection["input_identity"],
                            "setup_identity": selection["setup_identity"],
                            "product_identity": selection.get("product_identity"),
                            "image": selection.get("image"),
                        }
                    except Exception as error:
                        proofs.append({"family": family, "case": row["scenario_id"],
                                       "status": "INCOMPLETE", "error": str(error)})
                        print(f"PROOF {family} {row['scenario_id']} INCOMPLETE resolve {error}", flush=True)
                        continue
                elif not identities:
                    proofs.append({"family": family, "case": row["scenario_id"],
                                   "status": "INCOMPLETE", "error": "no matching performance identities"})
                    print(f"PROOF {family} {row['scenario_id']} INCOMPLETE missing identities", flush=True)
                    continue
                if getattr(args, "proofs", "selected") == "selected" and family != PROOF_FAMILY and (row.get("tier") or 1) > 10:
                    proofs.append({"family": family, "case": row["scenario_id"], "status": "SKIPPED",
                                   "reason": "selected-shape coverage uses compact/tier-1/10 proofs; large-tier proofs are not replayed under the 45s/59s ceiling"})
                    print(f"PROOF {family} {row['scenario_id']} SKIPPED selected-shape", flush=True)
                    continue
                result = verify_row(args, family, row, output, identities)
                proofs.append(result)
                _write(output / "verification-ledger.json", proofs)
                print(f"PROOF {family} {row['scenario_id']} {result.get('status')} wall={result.get('wall_seconds')}", flush=True)
        _write(output / "verification-ledger.json", proofs)

    terminal = {
        "issue": 54,
        "performance_cases": len(inventory["performance_cases"]),
        "proof_definitions": len(inventory["proof_cases"]),
        "mismatches": inventory["mismatches"],
        "performance_outcomes": {},
        "verification_outcomes": {},
    }
    for item in ledger:
        terminal["performance_outcomes"][item.get("status") or "unknown"] = (
            terminal["performance_outcomes"].get(item.get("status") or "unknown", 0) + 1
        )
    for item in proofs:
        terminal["verification_outcomes"][item.get("status") or "unknown"] = (
            terminal["verification_outcomes"].get(item.get("status") or "unknown", 0) + 1
        )
    _write(output / "terminal-assessment.json", terminal)
    print(json.dumps(terminal, sort_keys=True), flush=True)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, RuntimeError, TimeoutError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(1)
