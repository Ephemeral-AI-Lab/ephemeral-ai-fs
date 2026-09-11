#!/usr/bin/env python3
"""issue #114 analyzer: cost breakdown of dedup-cdc-scattered-100.

Reads only retained receipts.  No measurement, no product change.

Sources
  1. The retained #113 A/B receipts (24 samples/arm, interleaved C X X C):
       /Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/evidence/ab/{C,X}/dedup-cdc-scattered-100/block*/
  2. The issue #114 STEP 1 instrumentation probe:
       issue114/step1/probe.jsonl
  3. The secondary tier receipts (tiers 10/100/500):
       evidence/speed/{C,X}/scattered-*

Usage:  python3 analyze-scattered-100.py [--json]
"""
import argparse
import glob
import json
import os
import statistics as st
from pathlib import Path

ROOT = Path("/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms")
AB = ROOT / "evidence/ab"
SPEED = ROOT / "evidence/speed"
ARM_NAMES = {"C": "C (4096 B)", "X": "X (65536 B)"}


def sample_phase(row):
    """Decompose one sample row into the runner's own wall brackets."""
    sc = next((r for r in row["records"] if r.get("kind") == "sample-complete"), None)
    rowwin = next((r for r in row["records"] if r.get("kind") == "runtime-observation-window"), None)
    store = [r for r in row["records"] if r.get("kind") == "store-observation"]
    wall = row["wall_ns"]
    prep = row["preparation_wall_ns"]
    cmd = row["command_wall_ns"]
    clean = row["cleanup"]["wall_ns"]
    return {
        "wall_ns": wall,
        "preparation_wall_ns": prep,
        "command_wall_ns": cmd,
        "cleanup_wall_ns": clean,
        "unattributed_ns": wall - prep - cmd - clean,
        "pure_call_sum_ns": sc["pure_call_sum_ns"] if sc else None,
        "host_orchestration_ns": sc["host_orchestration_ns"] if sc else None,
        "orchestration_unattributed_ns": sc["orchestration_unattributed_ns"] if sc else None,
        "observation_window_ns": rowwin["elapsed_ns"] if rowwin else None,
        "store_before": next((r for r in store if r.get("phase") == "before"), None),
        "store_after": next((r for r in store if r.get("phase") == "after-initialize"), None),
    }


def load_ab(arm, case="dedup-cdc-scattered-100"):
    rows = []
    for block in sorted(glob.glob(str(AB / arm / case / "block*"))):
        if not os.path.isdir(block):
            continue
        for line in open(os.path.join(block, "perf.jsonl")):
            row = json.loads(line)
            if row.get("kind") != "sample":
                continue
            rec = sample_phase(row)
            rec["arm"] = arm
            rec["block"] = os.path.basename(block)
            rec["sample_index"] = row["sample_index"]
            rec["resources"] = row.get("resources", {})
            rows.append(rec)
    return rows


def load_speed(arm, case):
    """Secondary tier receipts are flat: evidence/speed/<arm>/<case>/perf.jsonl."""
    rows = []
    path = SPEED / arm / case / "perf.jsonl"
    if not path.exists():
        return rows
    for line in open(path):
        row = json.loads(line)
        if row.get("kind") != "sample":
            continue
        rec = sample_phase(row)
        rec["arm"] = arm
        rows.append(rec)
    return rows


def med(rows, key):
    vals = [r[key] for r in rows if r.get(key) is not None]
    return st.median(vals) if vals else None


def spread(rows, key):
    vals = [r[key] for r in rows if r.get(key) is not None]
    return (min(vals), st.median(vals), max(vals)) if vals else None


def report_ab(as_json=False):
    out = {}
    for arm in ("C", "X"):
        rows = load_ab(arm)
        out[arm] = {"n": len(rows), "phases": {}}
        wall = med(rows, "wall_ns")
        for key in ("wall_ns", "preparation_wall_ns", "command_wall_ns", "cleanup_wall_ns",
                    "unattributed_ns", "pure_call_sum_ns", "observation_window_ns"):
            m = med(rows, key)
            out[arm]["phases"][key] = {
                "median_ns": m,
                "pct_of_wall": (100.0 * m / wall) if (m and wall) else None,
                "min_ns": spread(rows, key)[0],
                "max_ns": spread(rows, key)[2],
            }
        # wall - pure gap, per sample (paired within arm, not across arms)
        gaps = [r["wall_ns"] - r["pure_call_sum_ns"] for r in rows if r["pure_call_sum_ns"]]
        out[arm]["wall_minus_pure_ns"] = {
            "median_ns": st.median(gaps), "min_ns": min(gaps), "max_ns": max(gaps),
        }
    if not as_json:
        for arm in ("C", "X"):
            d = out[arm]
            print(f"--- arm {ARM_NAMES[arm]}   n={d['n']} samples")
            w = d["phases"]["wall_ns"]["median_ns"]
            print(f"    {'wall_ns (median)':<32} {w/1e9:8.4f} s")
            for key in ("preparation_wall_ns", "command_wall_ns", "cleanup_wall_ns",
                        "unattributed_ns", "observation_window_ns", "pure_call_sum_ns"):
                p = d["phases"][key]
                print(f"    {key:<32} {p['median_ns']/1e6:8.2f} ms  {p['pct_of_wall']:5.2f}% of wall")
            g = d["wall_minus_pure_ns"]
            print(f"    wall_ns - pure_call_sum_ns       {g['median_ns']/1e9:8.4f} s  "
                  f"(min {g['min_ns']/1e9:.3f} max {g['max_ns']/1e9:.3f})")
    return out


def report_probe(as_json=False):
    path = ROOT / "issue114/step1/probe.jsonl"
    if not path.exists():
        return None
    rows = [json.loads(l) for l in open(path)]
    # iteration 1 is a cold-image warm-up: image inspect + first container create
    warm = [r for r in rows if r["iteration"] == 1]
    rows = [r for r in rows if r["iteration"] > 1]
    out = {"n": len(rows), "warmup_excluded": len(warm), "components": {}}
    for key in ("host_acquire_ns", "start_sample_ns", "host_sample_ns",
                "cgroup_snapshot_ns", "remove_ns", "remove_host_owned_ns"):
        vals = [r["observed_ns"].get(key, 0) for r in rows]
        out["components"][key] = {"median_ns": st.median(vals), "min_ns": min(vals), "max_ns": max(vals)}
    for key in ("runner_wall_ns", "runner_prep_wall_ns", "runner_cmd_wall_ns",
                "runner_cleanup_ns", "pure_call_sum_ns"):
        vals = [r[key] for r in rows if r.get(key)]
        out["components"][key] = {"median_ns": st.median(vals), "min_ns": min(vals), "max_ns": max(vals)}
    prep = out["components"]["runner_prep_wall_ns"]["median_ns"]
    out["preparation_shares"] = {
        "host_acquire": out["components"]["host_acquire_ns"]["median_ns"] / prep,
        "start_sample": out["components"]["start_sample_ns"]["median_ns"] / prep,
        "host_sample": out["components"]["host_sample_ns"]["median_ns"] / prep,
    }
    clean = out["components"]["runner_cleanup_ns"]["median_ns"]
    out["cleanup_shares"] = {
        "docker_rm": out["components"]["remove_ns"]["median_ns"] / clean,
        "remove_host_owned": out["components"]["remove_host_owned_ns"]["median_ns"] / clean,
    }
    wall = out["components"]["runner_wall_ns"]["median_ns"]
    res = (out["components"]["runner_wall_ns"]["median_ns"]
           - out["components"]["runner_prep_wall_ns"]["median_ns"]
           - out["components"]["runner_cmd_wall_ns"]["median_ns"]
           - out["components"]["runner_cleanup_ns"]["median_ns"])
    out["residue_ns"] = res
    out["residue_explained_by_cgroup"] = out["components"]["cgroup_snapshot_ns"]["median_ns"] / res
    if not as_json:
        print(f"--- STEP 1 probe (n={out['n']}, warm-up iterations excluded={out['warmup_excluded']})")
        for key, v in out["components"].items():
            print(f"    {key:<28} {v['median_ns']/1e6:8.2f} ms  "
                  f"(min {v['min_ns']/1e6:7.2f} max {v['max_ns']/1e6:7.2f})")
        print(f"    preparation: " + ", ".join(
            f"{k}={v*100:.1f}%" for k, v in out["preparation_shares"].items()))
        print(f"    cleanup:     " + ", ".join(
            f"{k}={v*100:.1f}%" for k, v in out["cleanup_shares"].items()))
        print(f"    residue {res/1e6:.2f} ms explained by cgroup_snapshot x2: "
              f"{out['residue_explained_by_cgroup']*100:.1f}%")
    return out


def report_tiers(as_json=False):
    out = {}
    for tier in (10, 100, 500):
        out[tier] = {}
        for arm in ("C", "X"):
            if tier == 100:
                # tier 100 is the retained interleaved A/B (24 samples/arm)
                rows = load_ab(arm, "dedup-cdc-scattered-100")
            else:
                rows = load_speed(arm, "scattered-%d" % tier)
            out[tier][arm] = {
                "n": len(rows),
                "pure_median_ns": med(rows, "pure_call_sum_ns"),
                "wall_median_ns": med(rows, "wall_ns"),
            }
    for tier in out:
        c, x = out[tier]["C"], out[tier]["X"]
        out[tier]["ratio_pure"] = x["pure_median_ns"] / c["pure_median_ns"]
        out[tier]["ratio_wall"] = x["wall_median_ns"] / c["wall_median_ns"]
    if not as_json:
        print("--- tier curve (X/C of the pure_call_sum timer)")
        for tier in sorted(out):
            d = out[tier]
            print(f"    tier {tier:>4}  C={d['C']['pure_median_ns']/1e6:8.2f} ms (n={d['C']['n']})  "
                  f"X={d['X']['pure_median_ns']/1e6:8.2f} ms (n={d['X']['n']})  "
                  f"ratio={d['ratio_pure']:.3f}   wall ratio={d['ratio_wall']:.3f}")
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    ab = report_ab(args.json)
    probe = report_probe(args.json)
    tiers = report_tiers(args.json)
    if args.json:
        print(json.dumps({"ab": ab, "probe": probe, "tiers": tiers}, indent=1))


if __name__ == "__main__":
    main()
