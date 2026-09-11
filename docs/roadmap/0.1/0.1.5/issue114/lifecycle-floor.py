#!/usr/bin/env python3
"""issue #114 — Direction 4: re-scope the benchmark metric.

WHY
    `wall_ns` is dominated by per-sample container lifecycle, not product work.
    Measured over the 387 retained A/B samples: preparation_wall_ns 816.9 ms +
    cleanup.wall_ns 435.8 ms = 1236.7 ms, near-constant across families whether
    the product timer is 11 ms or 4,762 ms.  On dedup-cdc-scattered-100 the
    timer is 14.5% (C) / 8.6% (X) of wall.  Any optimization loop steered by
    `wall_ns` is steering by docker create/start/rm.

WHAT THIS IS
    A DERIVED, READ-ONLY view.  It changes no receipt, no timer, no threshold,
    no verifier and no product code.  Every field it prints is already present
    in the retained `perf.jsonl`; this module only labels which part of `wall_ns`
    is harness lifecycle and which part can plausibly respond to a product change.

THE DECOMPOSITION
    wall_ns
      = lifecycle_ns      (harness: container spawn, teardown, cgroup observers)
      + product_window_ns (the time in which product work can actually move)
      + post_timer_ns     (in-process drain after the timer stops)

    lifecycle_ns    = preparation_wall_ns + cleanup.wall_ns + (wall residue)
    product_window_ns = command_wall_ns
    Within the command window the declared timer is `pure_call_sum_ns`; the rest
    is post-timer drain (observation window) plus command overhead.

DEFINITIONS
    harness_lifecycle_ns = preparation_wall_ns + cleanup_wall_ns + residue_ns
        residue_ns = wall_ns - preparation_wall_ns - command_wall_ns - cleanup_wall_ns
                   = the two cgroup_snapshot `docker exec` calls (~284 ms on C)
    product_window_ns    = command_wall_ns            (the only movable bracket)
    timer_ns             = the declared product timer  (pure_call_sum_ns)
    timer_share_of_wall  = timer_ns / wall_ns          (<-- the honest steer signal)
    timer_share_of_window= timer_ns / product_window_ns

USAGE
    python3 lifecycle-floor.py <case> [--arm C|X] [--json]
    python3 lifecycle-floor.py --all            # every retained A/B case
"""
import argparse
import glob
import json
import os
import statistics as st
from pathlib import Path

AB = Path("/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/evidence/ab")

TIMER_FALLBACKS = (
    "pure_call_sum_ns", "product_call_sum_ns", "elapsed_ns",
    "edit_commit_ns", "initialize_ns", "layerstack_init_ns",
    "complete_ns", "complete_lifecycle_ns", "execution_ns",
)


def declared_timer(row):
    """Return (name, ns) for the receipt's declared timer."""
    declared = row.get("identities", {}).get("timer")
    keys = (declared,) if declared else TIMER_FALLBACKS
    for record in reversed(row.get("records", [])):
        for key in keys:
            if isinstance(record.get(key), (int, float)):
                return key, int(record[key])
    return declared, None


def decompose(row):
    """Label the parts of one sample's wall_ns. Pure function of the receipt."""
    wall = row["wall_ns"]
    prep = row.get("preparation_wall_ns")
    cmd = row.get("command_wall_ns")
    clean = (row.get("cleanup") or {}).get("wall_ns")
    if None in (prep, cmd, clean):
        return None
    residue = wall - prep - cmd - clean
    name, timer = declared_timer(row)
    return {
        "wall_ns": wall,
        "harness_lifecycle_ns": prep + clean + residue,
        "preparation_ns": prep,
        "cleanup_ns": clean,
        "residue_ns": residue,
        "product_window_ns": cmd,
        "timer_name": name,
        "timer_ns": timer,
        "timer_share_of_wall": (timer / wall) if timer else None,
        "timer_share_of_window": (timer / cmd) if timer else None,
        "lifecycle_share_of_wall": (prep + clean + residue) / wall,
    }


def load(case, arm):
    rows = []
    for block in sorted(glob.glob(str(AB / arm / case / "block*"))):
        if not os.path.isdir(block):
            continue
        for line in open(os.path.join(block, "perf.jsonl")):
            row = json.loads(line)
            if row.get("kind") != "sample":
                continue
            rec = decompose(row)
            if rec:
                rec["arm"] = arm
                rec["case"] = case
                rows.append(rec)
    return rows


def summarize(rows):
    m = lambda k: st.median([r[k] for r in rows if r.get(k) is not None])
    return {
        "n": len(rows),
        "wall_ns": m("wall_ns"),
        "harness_lifecycle_ns": m("harness_lifecycle_ns"),
        "preparation_ns": m("preparation_ns"),
        "cleanup_ns": m("cleanup_ns"),
        "residue_ns": m("residue_ns"),
        "product_window_ns": m("product_window_ns"),
        "timer_ns": m("timer_ns"),
        "timer_name": rows[0].get("timer_name"),
        "timer_share_of_wall": m("timer_share_of_wall"),
        "timer_share_of_window": m("timer_share_of_window"),
        "lifecycle_share_of_wall": m("lifecycle_share_of_wall"),
    }


def print_case(case, arm, s):
    print(f"\n=== {case}  [arm {arm}]  n={s['n']}  timer={s['timer_name']}")
    w = s["wall_ns"]
    print(f"    wall_ns (median)              {w/1e6:10.2f} ms   100.0%")
    print(f"      harness lifecycle           {s['harness_lifecycle_ns']/1e6:10.2f} ms  "
          f"{100*s['lifecycle_share_of_wall']:5.1f}%   <- NOT movable by product change")
    print(f"        preparation (spawn)       {s['preparation_ns']/1e6:10.2f} ms")
    print(f"        cleanup (teardown)        {s['cleanup_ns']/1e6:10.2f} ms")
    print(f"        cgroup observers (x2)     {s['residue_ns']/1e6:10.2f} ms")
    print(f"      product window (command)    {s['product_window_ns']/1e6:10.2f} ms  "
          f"{100*s['product_window_ns']/w:5.1f}%   <- the only movable bracket")
    t = s["timer_ns"]
    if t:
        print(f"        of which declared timer   {t/1e6:10.2f} ms  "
              f"{100*s['timer_share_of_wall']:5.1f}% of wall / "
              f"{100*s['timer_share_of_window']:5.1f}% of window")
    print(f"    STEER SIGNAL: timer is {100*(s['timer_share_of_wall'] or 0):.1f}% of wall")
    if s["timer_share_of_wall"] is not None and s["timer_share_of_wall"] < 0.25:
        print("      -> wall_ns is NOT a product metric here; steer by the timer.")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("case", nargs="?")
    ap.add_argument("--arm", default=None)
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    out = {}
    if args.all:
        cases = sorted({os.path.basename(d) for a in ("C", "X")
                        for d in glob.glob(str(AB / a / "*")) if os.path.isdir(d)})
        for case in cases:
            for arm in ("C", "X"):
                rows = load(case, arm)
                if rows:
                    out[f"{case}/{arm}"] = summarize(rows)
        if args.json:
            print(json.dumps(out, indent=1))
            return
        ranking = sorted(out.items(), key=lambda kv: kv[1]["timer_share_of_wall"] or 0)
        print(f"{'case/arm':<48}{'timer_ms':>10}{'wall_ms':>10}{'life%':>8}{'timer/wall':>12}")
        for key, s in ranking:
            print(f"{key:<48}{s['timer_ns']/1e6:10.1f}{s['wall_ns']/1e6:10.1f}"
                  f"{100*s['lifecycle_share_of_wall']:7.1f}%{100*s['timer_share_of_wall']:11.1f}%")
        return

    if not args.case:
        ap.error("give a case or --all")
    arms = [args.arm] if args.arm else ["C", "X"]
    for arm in arms:
        rows = load(args.case, arm)
        if not rows:
            continue
        s = summarize(rows)
        out[f"{args.case}/{arm}"] = s
        if not args.json:
            print_case(args.case, arm, s)
    if args.json:
        print(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
