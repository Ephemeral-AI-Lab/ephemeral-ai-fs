#!/usr/bin/env python3
"""Generate release-notes/0.1.5/benchmark-closeout.md (report-only).

Per-case tables come from benchmark-performance.csv, which is itself derived from
docs/roadmap/0.1/0.1.5/issue120/final-report.md section 2 and verified against the
campaign receipts. Family prose summarises the published family reports. Nothing
is executed and no number is recomputed.
"""
import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "release-notes/0.1.5"

FAMILY_ORDER = [
    "payload_create_read", "dedup_workspace_reuse", "dedup_cross_file",
    "dedup_cdc_locality", "edit_length_preserving", "edit_length_changing",
    "edit_canonical_chunk_count", "init_namespace", "store_footprint",
    "tiny_file_churn", "namespace_mutation", "directory_construction_traversal",
    "workspace_change_locality", "dedup_branch_history", "git_tool_workflow",
    "mixed_load_bearing", "workspace_reliability",
]

FAMILY_NOTES = {
    "payload_create_read": (
        "All four `payload-create-*` cells **PASS and are faster than v0.1.3** "
        "(0.79–1.02×); the three random-read cells WARN (+5.9–13.3 ms) and one is "
        "reused from the fsync-qualified treatment."),
    "dedup_workspace_reuse": (
        "The exact/local reuse tiers are the strongest v0.1.3 improvements in the "
        "campaign: `exact-100` 0.62× and `local-100` 0.66×. Eight PASS, five WARN, "
        "one reused."),
    "dedup_cross_file": (
        "All eight fresh cells WARN against v0.1.3 (ratios 1.46–3.35×, +11 ms to "
        "+691 ms); two reused. These ratios compare the v0.1.5 ordinary path "
        "(authenticated CAS + CDC + pack assembly) against pre-authentication "
        "v0.1.3 references — #112 context, not a gate."),
    "dedup_cdc_locality": (
        "Two PASS (`insert-10`, `delete-10`), seventeen WARN with ratios 1.04–3.45×; "
        "`scattered-500` carries the largest fresh aggregate (+943.03 ms, still "
        "under the mandatory >1 s diagnosis threshold) and `scattered-100` is the "
        "campaign's worst ratio (3.45×) with an **≈452 ms unattributed gap** "
        "between its timer and command window (#114). One reused, plus the "
        "proof-only `dedup-cdc-boundaries-proof` PASS (1.74 s)."),
    "edit_length_preserving": (
        "One public SDK range-edit plus its Commit per cell (`edit_commit_ns`, "
        "ops=1). Eleven fresh, one reused from the `6693224e…` affected-rerun; all "
        "24 registered targets PASS. Nine WARN cells sit at +3.3–7.3 ms per-op, "
        "every one under the 10 ms/op hard cap with no n3 comparator at n=1."),
    "edit_length_changing": (
        "Thirty fresh (12 PASS / 18 WARN, +3.0–6.9 ms) plus two reused from the "
        "`6693224e…` affected-rerun. All registered targets PASS."),
    "edit_canonical_chunk_count": (
        "Twelve fresh; five PASS, seven WARN (+3.3–7.3 ms), all under the hard cap. "
        "The candidate is within or below the #104 uncompacted band on 22 of these "
        "24 edit cells (diagnostic context)."),
    "init_namespace": (
        "All four tiers WARN against v0.1.3 (2.53–3.22×). `namespace-100000` is the "
        "campaign's VERIFIED_COLD member: 100,000 files, 125,169 expected pages, "
        "**0 resident** after eviction, metadata VERIFIED, acquisition 25.942 s "
        "reported separately and inside the 31.615 s complete envelope. Its 2.7 s "
        "absolute target is **owner-waived** ([waivers](../0.1.5/waivers.md) §2) and "
        "its 4.3975 s measurement stands as a TARGET_MISS. The catalogued >1 s "
        "diagnosis: +1.794 s is one Init of 100,000 files on the ordinary path "
        "(authenticated CAS + pack assembly + delta encoding); the same cell "
        "measured 4.841 s on #104 (candidate −9.1 % versus it)."),
    "store_footprint": (
        "Six WARN rows (timers 1.44–2.55× v0.1.3) with footprints matching #104 "
        "within ±0.04 %. **Recorded reuse deviation:** the two cells that appeared "
        "in the #120 reuse list were re-collected fresh because #107 changes "
        "exactly their measured quantity; `unique-100000` allocated 530,358,272 → "
        "**520,142,848 B (−1.9 %)** — reusing the old number would have misstated "
        "the candidate by 10 MB. Its construction timer 5.397 s is +36 % versus "
        "fsync-qualified: the #107 pack-row UPDATE cost on one giant commit."),
    "tiny_file_churn": (
        "Nineteen fresh (4 PASS / 15 WARN, +3.2–679 ms) plus one reused. The "
        "registered Tier-1 gate `tiny-create100 < 1 s` **PASSES at 79 ms**."),
    "namespace_mutation": (
        "Four fresh WARN rows (+8.2–92.1 ms, ratios 1.24–1.50×); no aggregate "
        "exceeds 1 s."),
    "directory_construction_traversal": (
        "Eleven fresh (2 PASS / 9 WARN, +5.4–189.7 ms) plus one reused; the largest "
        "fresh delta is +189.73 ms, below the >1 s diagnosis threshold."),
    "workspace_change_locality": (
        "Twelve fresh (4 PASS / 8 WARN) plus four reused from the fsync-qualified "
        "treatment. Highlight: `distributed-sdk-edit-100` **0.63×** and `-500` "
        "**0.23×** — the #116 bounded-pending win. The two reused >1 s diagnoses "
        "(`dense-rewrite-100` +1.40 s, `-500` +4.81 s) are the batched-fsync write "
        "path plus authenticated pack admission, measured on the reused treatment."),
    "dedup_branch_history": (
        "Seven PASS, twelve WARN, and the campaign's single FAIL. The 500-tier "
        "history cells (+0.64–1.38 s) match 500 × the per-iteration #116/#107 "
        "mechanism costs measured in the unrelated-500 RCA. `unrelated-1/10/100` "
        "are 0.52–0.90× v0.1.3 (faster) while `unrelated-500` is **0.89×** yet "
        "still misses its own absolute gate — see the bug ledger below."),
    "git_tool_workflow": (
        "Three fresh WARN rows (1.06×/+18 ms, 1.15×/+94 ms, 1.51×/+959 ms) plus "
        "`git-tool-500` reused from fsync-qualified (8.803 s, 1.88× v0.1.3; the "
        "reused row's >1 s diagnosis is the per-iteration mechanism cost plus "
        "authentication/pack work on the object path)."),
    "mixed_load_bearing": ("Three fresh (1 PASS / 2 WARN) plus one reused; largest fresh delta +121.67 ms."),
    "workspace_reliability": (
        "Twenty-seven runnable proofs **PASS** (2.1–9.0 s) and one is "
        "`NOT_RUN_OPTIONAL`. These are proof-only selections: they carry no "
        "performance timer and none is reported as a speed result."),
}


def rows():
    with (OUT / "benchmark-performance.csv").open() as stream:
        return list(csv.DictReader(stream))


def proof_rows():
    with (OUT / "benchmark-verification.csv").open() as stream:
        return list(csv.DictReader(stream))


def esc(text):
    return text.replace("|", "\\|")


def table(records):
    lines = [
        "| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |",
        "|---|---|---|---:|---:|---:|---:|---:|---|---|---|",
    ]
    for row in records:
        source = "fresh" if row["collection"] == "fresh" else "reused: " + row["producing_treatment"].split(" (")[0]
        lines.append(
            f"| `{row['case']}` | {row['kind']} | {row['timer'] or '—'} | "
            f"{row['candidate_value'] or '—'} | {row['reference_v0_1_3'] or '—'} | "
            f"{row['ratio'] or '—'} | {row['delta_per_op'] or '—'} | {row['ops'] or '—'} | "
            f"{esc(row['disposition'])} | {row['proof_disposition']} | {source} |")
    return "\n".join(lines)


def proof_table(records):
    lines = [
        "| Selection | Kind | Proof status | Proof wall (s) | Omissions | Evidence |",
        "|---|---|---|---:|---|---|",
    ]
    for row in records:
        lines.append(
            f"| `{row['case']}` | {row['selection_kind']} | {row['proof_status']} | "
            f"{row['proof_wall_seconds'] or '—'} | {esc(row['omissions']) or '—'} | "
            f"`{row['evidence']}` |")
    return "\n".join(lines)


def main():
    performance = rows()
    proofs = proof_rows()
    by_family = {}
    for row in performance:
        by_family.setdefault(row["family"], []).append(row)
    proof_only = [row for row in proofs if row["selection_kind"] == "proof-only"]
    proof_only_by_family = {}
    for row in proof_only:
        proof_only_by_family.setdefault(row["family"], []).append(row)

    fresh = sum(1 for row in performance if row["collection"] == "fresh")
    reused = len(performance) - fresh
    pass_n = sum(1 for row in performance if row["disposition"] == "PASS")
    warn_n = sum(1 for row in performance if row["disposition"] == "WARN")
    fail_n = sum(1 for row in performance if row["disposition"] == "FAIL")
    proof_pass = sum(1 for row in proofs if row["proof_status"] == "PASS")
    proof_opt = sum(1 for row in proofs if row["proof_status"] == "NOT_RUN_OPTIONAL")

    parts = [f"""# v0.1.5 benchmark closeout — every family and case

> **Status:** LayerFS 0.1.5 release record. Generated report-only from the #120
> campaign; **no benchmark or proof was run for this release**. The per-case
> values and dispositions are the published ones from
> `docs/roadmap/0.1/0.1.5/issue120/final-report.md` §2, re-verified row by row
> against the immutable receipts and expanded with the receipt facts
> ([benchmark-performance.csv](benchmark-performance.csv),
> [benchmark-verification.csv](benchmark-verification.csv)). Dispositions are
> copied, never recomputed: FAIL stays FAIL, WARN stays WARN, and a reused value
> is never presented as a candidate measurement.

## Protocol

One complete public sample per registered selection (`--perf-fast
--collection-mode`, 300/310/600 s allowances), seed 1, repetition 1, per the
frozen #120 campaign contract; one independent identity-pinned proof per fresh
selection. The runner and verifier serialized on the measurement lock and every
receipt is immutable under `benchmark-results/host-store/issue120/`. Reused
rows carry their source run's contract and name their producing treatment.

**Disposition convention (ordinary comparative cells):** candidate ≤ reference,
or Δ/op under the 3 ms wall floor → PASS (values recorded either way);
beyond-floor non-Tier-2 material (at n=1 no alternating-pair comparator exists,
so Tier-2 condition 2 cannot hold) → WARN with ratio, absolute and per-op delta;
Tier-2 material (all three conditions) → FAIL. Registered harness targets are
Tier-1 and are PASS/FAIL/OWNER-WAIVED directly.

**References:** published v0.1.3 checkpoint single samples with an **undeclared
cache profile**, so every ratio is historical context, never a paired claim.

## Tallies

| Tally | Value |
|---|---|
| Registered performance selections | **198/198 terminal** = {fresh} fresh + {reused} reused |
| Fresh dispositions | **{pass_n} PASS / {warn_n} WARN / {fail_n} FAIL** |
| Proof-only selections | **29/29 terminal** = 28 PASS + 1 NOT_RUN_OPTIONAL |
| Proof dispositions (227 registered selections) | {proof_pass} PASS + 16 reused + {proof_opt} NOT_RUN_OPTIONAL |
| Fresh independent proofs | **182/182 PASS**; every cleanup PASS; no S0/S1, no H-class finding |
| Verified-cold member | `namespace-100000` (0/125,169 resident pages, metadata VERIFIED, acquisition inside the envelope) |
| Native gate on the measured tree | `tools/test-fast.sh` PASS — 530 tests/benchmarks, 118 s, exit 0 |
| Cache contract | warm-prepared-reuse with a clone per sample and the cache identity in the receipt, except the VERIFIED_COLD member; reused rows inherit their source run's contract |

## Family → per-case tables

The tables below list **every** registered selection. `Source` is `fresh` for a
candidate measurement on `c55daf13…` @ `1ff1f2ddd`, or the producing treatment
for a reused value.
"""]

    for index, family in enumerate(FAMILY_ORDER, start=1):
        records = by_family.get(family, [])
        proof_records = proof_only_by_family.get(family, [])
        performance_records = [row for row in records if row["kind"] == "performance"]
        total = len(records) + len(proof_records)
        parts.append(f"\n### {index}. `{family}` — {total} registered selections\n")
        if family in FAMILY_NOTES:
            parts.append("\n" + FAMILY_NOTES[family] + "\n")
        if performance_records:
            parts.append("\n" + table(performance_records) + "\n")
        if proof_records:
            parts.append("\n" + proof_table(proof_records) + "\n")

    parts.append(f"""
## Reused evidence (cited, never claimed as new)

| Item | Producing treatment | Value | Disposition |
|---|---|---|---|
| Ordinary full157 stride-1 construction | final treatment `b5f089eb…` | complete command 763.218 s, 157/157 steps PASS | REUSED (mechanism unchanged; `3e308a8f2` behavior-neutral) |
| full157 same-Store verification | final treatment | 735.749 s; 157 states / 904,143 entries / 4,936,693,030 B; all oracles PASS | REUSED |
| Physical census | final treatment | allocated 83,951,616 B; apparent 82,583,552 B; pack rows 1,058; `store_sha256 88b4fe70…` | REUSED |
| Git157 read-only control | final treatment | 157 checkpoints, mapping verified; allocation-layout WARN retained (56,197,120 vs recorded 56,373,248 B) | REUSED with its WARN |
| Historical access 11 performance + 11 proofs | final treatment | outer walls 2.525–3.232 s, verification 2.48–2.87 s, all 22 PASS inside the unwaived 15 s envelope; cleanups PASS | REUSED |
| Default-budget K32000 route | final treatment | COMPLETE in one Commit; 32,000 edits / 92,821 pieces / charge 2,048,000 under the unchanged 2 MiB budget; independent verification PASS (68.576 s) | REUSED; its historical-family `TARGET_MISS` is the recorded [waiver](../0.1.5/waivers.md) §4 WARN |
| K6000 boundary + default-budget frontier proof | final treatment | K6000 PASS (14.61 s); frontier PASS at exactly 2B (batch=15873 count=31746 flushes=2), 4B and 8B | REUSED |
| 16 registered campaign cells | `6693224e…` affected-rerun (3 SDK-edit cells) and `fsync-qualified` `440ae2c4…` (13 cells) | see the per-case tables; each row names its treatment | REUSED-FROM |

Deviation recorded honestly: the two `store_footprint` cells on the #120 reuse
list were **re-collected fresh** because #107 changes exactly their measured
quantity; the old receipts remain cited as history. One archived-binary custody
observation is also retained: `binary-archive/6693224e…/identity.json` records a
product seal that differs from the #118 affected-rerun record although the binary
bytes hash to the same `6693224e…`; the diagnostic attribution run used the
archive copy with its seal-consistent image, and because the bytes are identical
that attribution is byte-determined.

## Bug ledger

| Severity | Evidence | Reproducer | Root cause | Fix | Impact-set re-run | Disposition |
|---|---|---|---|---|---|---|
| **S2** (Tier-1 gate miss) | `performance/dedup_branch_history/dedup-history-unrelated-500-mixed-v2/perf.jsonl` — 16.107 s vs `< 15 s`; complete command 19.057 s; cleanup PASS | the campaign receipt plus the diagnostic run `diagnostics/unrelated-500-on-6693224e/perf.jsonl` (15.772 s on the #116-without-#107 binary) | 500 × (exec 12.42→15.30 ms, commit 15.14→16.88 ms): #116 bounded pending ≈ +1.98 s (dominant), #107 pack coalescing ≈ +0.33 s; introduced when #116 landed and undetected because #118's affected-rerun covered only the 3 SDK-edit cells | none (not a minimal-fix candidate) | none | **FAIL — unrepaired**, dispositioned by the owner [waiver](../0.1.5/waivers.md) §1; the cell is 0.89× v0.1.3 |

No S0, no S1 and no H-class finding occurred: 182/182 fresh samples COMPLETE,
182/182 fresh proofs PASS, 28/28 runnable proof-only proofs PASS, every cleanup
PASS, no custody/identity failure and no resource-bound breach.

## Campaign-wide distribution (context for #112 — never a gate)

Across the 198 comparable registered cells: **139 of 198 cells ≥15 % slower**
than published v0.1.3, median ratio **1.34×**, total added time **+33.115 s**.
Worst ratios: `dedup-cdc-scattered-100` 3.45×, `dedup-cross-file-unique-100`
3.35×, `namespace-100` 3.22×. Largest absolute deltas: reused
`dense-rewrite-500` +4.81 s, fresh 500-tier history cells +0.64–1.38 s,
`git-tool-100` +0.96 s, `scattered-500` +0.94 s. Faster than v0.1.3:
`payload-create-*` (0.79–1.02×), `dedup-workspace exact/local-100` (0.62–0.66×),
`distributed-sdk-edit-100/500` (0.63×/0.23×), `unrelated-1/10/100` (0.52–0.90×).

Against the earlier #104 baseline the candidate's median ratio is **0.993×** with
38 fresh cells ≥15 % faster (best: `workspace-distributed-sdk-edit-500`
4.131 → 0.666 s = 0.16×) and 29 fresh cells ≥15 % slower — consistent with the
#116/#107 fixed per-iteration costs on ms-scale cells.

## Explicit gaps and omissions

1. `workspace-sustained-600s-compact-v2-proof` — **NOT_RUN_OPTIONAL**, excluded
   by the frozen campaign declaration (`long_test_exclusion`): optional long
   test, **no endurance qualification in v0.1.5**. The other five extended
   reliability members and all 27 runnable reliability proofs ran and PASSed.
2. `dedup-history-unrelated-500-mixed-v2` — FAIL, dispositioned by waiver
   ([waivers](../0.1.5/waivers.md) §1).
3. The `store_footprint` reuse deviation recorded above.
4. The archived-binary seal observation recorded above.
5. `namespace-100000`'s 2.7 s absolute cold target — TARGET_MISS under the
   carried-forward owner waiver ([waivers](../0.1.5/waivers.md) §2).
6. The K32000 historical-family `TARGET_MISS` — reporting-only target recorded
   as WARN ([waivers](../0.1.5/waivers.md) §4).

## What this closeout does not say

It does not claim an all-gates pass, a speedup, a storage-target pass or
endurance. It does not present a reused value as a measurement of the candidate's
bytes, and it does not convert the FAIL, the 125 WARNs, the waived targets or the
unmet #108/#112/#114 gates into passes. Owner acceptance retires optimization
scope; the classifications above are unchanged.
""")

    (OUT / "benchmark-closeout.md").write_text("\n".join(parts))
    print("wrote benchmark-closeout.md")


if __name__ == "__main__":
    main()
