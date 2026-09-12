#!/usr/bin/env python3
"""Derive the v0.1.5 release CSVs (report-only) from the #120 final report table.

Reads docs/roadmap/0.1/0.1.5/issue120/final-report.md section 2, verifies every
receipt path it names against the local evidence tree, and cross-checks each
fresh performance value against its perf.jsonl summary. Writes
release-notes/0.1.5/benchmark-performance.csv and benchmark-verification.csv.
No benchmark, proof or test is executed.
"""
import csv
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPORT = ROOT / "docs/roadmap/0.1/0.1.5/issue120/final-report.md"
OUT = ROOT / "release-notes/0.1.5"
ISSUE120 = "benchmark-results/host-store/issue120"
ISSUE118 = "benchmark-results/host-store/issue118/20260912"
TREATMENTS = {
    "fsync-qualified": {
        "label": "fsync-qualified 440ae2c4… (commit f8fa59fab, pre-#107/#116)",
        "sha256": "440ae2c4ce03741676a2845becb9b1661dde341079cc62faad0998847d33a4e5",
        "root": f"{ISSUE118}/remaining-shared",
    },
    "6693224e": {
        "label": "6693224e… (commit 2dbc75ecb, +#116, pre-#107)",
        "sha256": "6693224ee0939a7b6e18489259cdd08ab380a1a0f2877d8a4c6caf2334abafc1",
        "root": f"{ISSUE118}/affected-rerun",
    },
}
AFFECTED_PROOF = {"overwrite-middle-4k-on-1mib-ops-1": "overwrite-proof", "insert-middle-4k-on-1mib-ops-1": "insert-middle-4k-on-1mib-ops-1-proof", "delete-middle-4k-on-1mib-ops-1": "delete-middle-4k-on-1mib-ops-1-proof"}

UNIT_NS = {"ns": 1, "ms": 1_000_000, "s": 1_000_000_000}


def parse_ns(text):
    text = text.strip().replace(",", "")
    match = re.fullmatch(r"([0-9.]+)\s*(ns|ms|s)", text)
    if not match:
        return None
    return int(round(float(match.group(1)) * UNIT_NS[match.group(2)]))


def display_tolerance(text):
    """Half a unit in the last digit the report displays, in ns."""
    text = text.strip().replace(",", "")
    match = re.fullmatch(r"([0-9.]+)([0-9])\s*(ns|ms|s)", text) or re.fullmatch(
        r"([0-9]+)\.([0-9]+)\s*(ns|ms|s)", text)
    if not match:
        return 1
    decimals = len(match.group(2))
    unit = UNIT_NS[match.group(3)]
    return 0.5 * (10 ** -decimals) * unit + 1


def split_row(line):
    cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
    return cells


def table_rows():
    rows = []
    inside = False
    for line in REPORT.read_text().splitlines():
        if line.startswith("| Family | Selection | Kind |"):
            inside = True
            continue
        if inside:
            if not line.startswith("|"):
                break
            cells = [clean(cell) for cell in split_row(line)]
            if cells[0].startswith("---") or cells[0] == "":
                continue
            rows.append(cells)
    return rows


def clean(text):
    return text.replace("**", "").replace("`", "").strip()


def evidence_for(family, case, disposition):
    """Return (perf_path, proof_path, source_class, treatment_label, treatment_sha)."""
    if "REUSED-FROM" in disposition:
        key = "6693224e" if "affected-rerun" in disposition else "fsync-qualified"
        treatment = TREATMENTS[key]
        perf = Path(f"{treatment['root']}/{case}/perf.jsonl")
        proof = Path(f"{treatment['root']}/{case}-proof/verification.json")
        if not (ROOT / proof).exists():
            proof = Path(f"{treatment['root']}/{AFFECTED_PROOF.get(case, '')}/verification.json")
        return perf, proof, "reused-issue118-receipt", treatment["label"], treatment["sha256"]
    perf = Path(f"{ISSUE120}/performance/{family}/{case}/perf.jsonl")
    proof = Path(f"{ISSUE120}/verification/{family}/{case}/verification.json")
    return perf, proof, "fresh-campaign-receipt", "c55daf13… @ 1ff1f2ddd", "c55daf13e372331a5ab6dbd465ece351a55923831c45864325ac46b1508fa295"


def perf_receipt(path):
    lines = (ROOT / path).read_text().splitlines()
    header = json.loads(lines[0])
    sample = json.loads(lines[1])
    summary = json.loads(lines[2])
    return header, sample, summary


def main():
    rows = table_rows()
    assert len(rows) == 227, f"expected 227 registered selections, found {len(rows)}"

    performance = []
    verification = []
    missing = []
    mismatches = []

    for cells in rows:
        (family, case, kind, timer, candidate, reference, ratio, delta, ops,
         disposition, proof) = cells
        disposition = clean(disposition)
        proof = clean(proof)
        perf_path, proof_path, source_class, treatment_label, treatment_sha = evidence_for(
            family, case, disposition)

        if kind == "performance":
            candidate_ns = parse_ns(candidate)
            reference_ns = parse_ns(reference) if reference != "—" else None
            receipt_ns = ""
            receipt_note = ""
            if source_class == "fresh-campaign-receipt":
                if not (ROOT / perf_path).exists():
                    missing.append(str(perf_path))
                else:
                    header, sample, summary = perf_receipt(perf_path)
                    receipt_ns = summary["median_ns"]
                    if abs(receipt_ns - candidate_ns) > display_tolerance(candidate):
                        mismatches.append(
                            f"{family}/{case}: table {candidate_ns} vs receipt {receipt_ns}")
                    receipt_note = (
                        f"timer={summary['timer']}; completion={sample['completion_status']}; "
                        f"command_wall_ns={sample['command_wall_ns']}; "
                        f"cleanup={sample['cleanup'].get('status')}; "
                        f"target_status={sample.get('historical_product_target_status')}")
            else:
                if not (ROOT / perf_path).exists():
                    missing.append(str(perf_path))
            performance.append({
                "family": family,
                "case": case,
                "kind": kind,
                "timer": timer,
                "candidate_value": candidate,
                "candidate_ns": candidate_ns,
                "candidate_receipt_ns": receipt_ns,
                "reference_v0_1_3": reference,
                "reference_v0_1_3_ns": reference_ns,
                "reference_profile": "undeclared (historical context, never a paired claim)",
                "ratio": ratio,
                "delta_per_op": delta,
                "ops": ops,
                "disposition": disposition,
                "proof_disposition": proof,
                "collection": "fresh" if source_class == "fresh-campaign-receipt" else "reused",
                "source_class": source_class,
                "producing_treatment": treatment_label,
                "producing_binary_sha256": treatment_sha,
                "evidence": str(perf_path),
                "verification_evidence": str(proof_path),
                "receipt_facts": receipt_note,
            })

        # one proof-disposition row per registered selection (198 + 29 = 227)
        if proof == "reused":
            status = "REUSED"
            evidence = str(proof_path)
            wall = ""
            omissions = "proof reused from the producing treatment; not re-run on the candidate"
        elif proof.startswith("exception"):
            status = "NOT_RUN_OPTIONAL"
            evidence = f"{ISSUE120}/verification/{family}/{case}/exception.json"
            wall = ""
            omissions = ("Optional long test under the frozen campaign declaration "
                         "(long_test_exclusion); unexecuted, no endurance qualification")
        else:
            status = proof
            evidence = str(proof_path)
            wall = ""
            omissions = ""
            if (ROOT / proof_path).exists():
                record = json.loads((ROOT / proof_path).read_text())
                wall = record.get("wall_seconds", "")
                omissions = "; ".join(record.get("omissions", []))
                if record.get("status") != proof:
                    mismatches.append(f"{family}/{case}: proof table {proof} vs receipt {record.get('status')}")
            else:
                missing.append(str(proof_path))

        verification.append({
            "family": family,
            "case": case,
            "selection_kind": kind,
            "proof_status": status,
            "proof_wall_seconds": wall,
            "omissions": omissions,
            "source_class": source_class,
            "producing_treatment": treatment_label,
            "producing_binary_sha256": treatment_sha,
            "evidence": evidence,
            "performance_evidence": str(perf_path) if kind == "performance" else "",
        })

    if missing:
        raise SystemExit("missing evidence paths:\n" + "\n".join(missing))
    if mismatches:
        raise SystemExit("value mismatches:\n" + "\n".join(mismatches))

    OUT.mkdir(parents=True, exist_ok=True)
    with (OUT / "benchmark-performance.csv").open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(performance[0]))
        writer.writeheader()
        writer.writerows(performance)
    with (OUT / "benchmark-verification.csv").open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(verification[0]))
        writer.writeheader()
        writer.writerows(verification)

    fresh = sum(1 for row in performance if row["collection"] == "fresh")
    reused = len(performance) - fresh
    pass_n = sum(1 for row in performance if row["disposition"] == "PASS")
    warn_n = sum(1 for row in performance if row["disposition"] == "WARN")
    fail_n = sum(1 for row in performance if row["disposition"] == "FAIL")
    proof_pass = sum(1 for row in verification if row["proof_status"] == "PASS")
    proof_opt = sum(1 for row in verification if row["proof_status"] == "NOT_RUN_OPTIONAL")
    proof_reused = sum(1 for row in verification if row["proof_status"] == "REUSED")
    print(json.dumps({
        "performance_rows": len(performance), "verification_rows": len(verification),
        "fresh": fresh, "reused": reused, "PASS": pass_n, "WARN": warn_n, "FAIL": fail_n,
        "proof_PASS": proof_pass, "proof_REUSED": proof_reused, "proof_NOT_RUN_OPTIONAL": proof_opt,
    }, indent=2))


if __name__ == "__main__":
    main()
