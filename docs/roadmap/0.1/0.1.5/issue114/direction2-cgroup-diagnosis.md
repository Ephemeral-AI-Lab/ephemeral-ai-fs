# Direction 2 — cheapen the observation overhead: measured diagnosis

Issue #114, recommended ordering step 2. **No product change. No receipt, timer,
threshold or verifier change.** This step is diagnosis only; the change it proposes
is specified in §5 and is *not* implemented here.

## 1. Question
The two `cgroup_snapshot` calls account for 283.6 ms = 13.4% of `wall_ns` on
`dedup-cdc-scattered-100`. Can that be reduced?

## 2. Method
`issue114/probe-cgroup-cost.py` starts one throwaway container via the harness's own
`runtime.start_sample`, then times, per iteration (n=15), the current implementation
against reduced variants, under the measurement lock. Plus a call-counting wrapper
around `runtime.run` over 6 real samples to enumerate every docker CLI invocation.

## 3. Where the cost actually is (n=15, medians)

| variant | ms |
|---|---:|
| **A. current** `docker exec sh -c 'cat 5 files'` | **133.39** |
| B. `docker exec <id> true` (no I/O at all) | 125.42 |
| C. `docker exec <id> cat /sys/fs/cgroup/cpu.stat` (1 file, no shell) | 125.36 |
| D. one exec, all five files | 128.33 |
| E. `docker inspect --format` (host-side, no exec) | 60.45 |
| F. `docker version` (host-side CLI floor) | 59.32 |

Derived:
- **Actual payload/read cost inside the exec = D − B = 2.91 ms = 2.3%.**
- Exec round-trip over a host-side call = B − E = 64.97 ms.
- Docker CLI process floor = 59.32 ms; the API call itself ≈ 1.13 ms.
- Host cgroup visible: **False** — a zero-exec host read is not possible on this topology.

**Prediction stated before running** — "`exec true` is ≥60% of the current cost because the
round-trip, not the read, dominates": **confirmed**, 125.42/133.39 = **94.0%**.

**Conclusion: the read is free; the `docker exec` round-trip is the entire cost.** Making
the in-container command cheaper buys ≤2.3%. There is no win available inside
`cgroup_snapshot`'s body.

## 4. The real lever: invocation *count*

Counting every `docker` CLI call one sample actually makes (wrapper around
`runtime.run`, real samples, `--setup fresh`, arm C image):

| verb | n | total ms | mean ms |
|---|---:|---:|---:|
| `exec` | 4 | 536.3 | 134.1 |
| `image` | 2 | 111.7 | 55.8 |
| `create` | 1 | 75.3 | 75.3 |
| `start` | 1 | 223.2 | 223.2 |
| `inspect` | 1 | 119.7 | 119.7 |
| `rm` | 1 | 431.2 | 431.2 |
| **total** | **10** | **1,497.5** | |

`wall_ns` 2,390.6 ms → **docker CLI = 62.6% of wall.**

Reproduced over 6 samples: **9–10 invocations, median 1,431.2 ms, 63.2% of wall.**
The count varies (9 vs 10) because the readiness poll can iterate, so the **poll is a
real uncontrolled variable**, not a constant.

Of the 4 `exec` calls: 2 are `cgroup_snapshot`, 1 is the readiness probe, 1 is the
capability read. Only **one** readiness iteration occurred in this sample — the count
above is a best case.

## 5. Recommendation (NOT implemented — needs protocol sign-off)

**Do not rewrite `cgroup_snapshot`'s body — it is 2.3% of its own cost.** Three
directions, ordered by (value ÷ risk):

1. **Fold the two `cgroup_snapshot` execs into existing calls.** `before` sits directly
   after `preparation_wall_ns` and `after` directly after `command_wall_ns`; both are
   adjacent to an `inspect`/`exec` already being made. Merging removes up to 2 of 10
   invocations ≈ **250 ms/sample** (≈12% of wall on this case).
   *Risk: low — same data, fewer round-trips.* **Caveat: it shifts the two snapshots
   relative to the command bracket. The `before`/`after` semantics must be re-argued,
   and the resulting counters must be shown to be unchanged within noise.**

2. **Cache the image identity.** `docker image inspect` is called twice per sample
   (111.7 ms). The image cannot change within a run. Memoising per process removes
   1 invocation ≈ **56 ms/sample**. *Risk: low, but it removes a per-sample staleness
   check that may be load-bearing for the "stale host/image/source identity" guard.*

3. **Cut readiness-poll latency.** `start_sample`'s poll costs one `exec` per iteration
   (134.1 ms each) and iterated at least once here. *Risk: medium — changing the
   readiness criterion weakens a correctness gate.*

**Not recommended:** container pooling / warm reuse. It would remove the largest block
but breaks the "fresh no-mount container" isolation the protocol depends on
(`docs/general/benchmark_rules.md`; the Docker prohibition in `benchmark/AGENTS.md`).

## 6. Honest bound

Even the full win (1 + 2 + 3 ≈ 300–400 ms/sample) is **measurement-side**. It reduces
the cost of *observing*, not the product's work, and on `dedup-cdc-scattered-100` it
would not move the 306.22 ms timer at all. Its value is (i) higher sample throughput,
and (ii) **making `wall_ns` less dominated by docker** — which is exactly what
Direction 4 was about: on `overwrite-middle-4k-on-500mib-ops-1` the timer is 0.3% of
wall, and ~0.6 s of that wall is docker CLI startup.

## 7. Falsifiable test for any resulting change
> Merging the two `cgroup_snapshot` calls must reduce per-sample `preparation +
> command + cleanup` residue by ≥200 ms while leaving every reported counter
> (`command_window_cpu_ns`, `sample_container_lifetime_peak_bytes`,
> `memory_current_bytes`, `swap_current_bytes`, `oom_kill_delta`) unchanged within
> the retained spread.

## 8. Receipts
- `issue114/step2/cgroup-probe.jsonl` — n=15 variant timings
- `issue114/probe-cgroup-cost.py` — the probe
- call-count wrapper: `/tmp/count-docker5.py` (scratch; 6 samples)
