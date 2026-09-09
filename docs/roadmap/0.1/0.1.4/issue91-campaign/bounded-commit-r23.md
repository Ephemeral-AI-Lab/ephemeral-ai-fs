# R23 bounded commit coalescing experiment

The user explicitly approved coalescing SQLite commits up to the existing public
ceilings of 8191 objects and ADMISSION_BATCH_BYTES (4 MiB minus one byte).
This supersedes the R6 ordinary transaction freeze for this experiment only.

Build on the provisional fresh-store negative filter. Keep physical preparation
batches at 512 objects/512 KiB, release their buffers after insertion, and preserve
4 KiB pages, exact native encoding, dependency order and imported library source,
versions and features. Enable coalescing only for native Init; other callers keep immediate commits.
Group already inserted physical batches in one bounded SQLite transaction; do not accumulate their canonical or encoded buffers.

Advance the publication epoch at each physical insertion, because same-connection
reads see that insertion before COMMIT. Commit before exceeding either public
ceiling, at final publication (including an empty final batch), and before the
workspace admission boundary hands off to root staging. Abort any open cohort
before existing rollback cleanup. Report actual commits and cohort maxima.

Use the existing host-only namespace100000 performance lane and shared lock.
Compare against the archived filter-only binary in r19-fast/build-011-filter-pair.
Retain commands, source patch, binary identity and all observations. No verification
or full157 during optimization. SQLite journal/cache memory may grow even while
physical preparation bounds remain fixed; report measured RSS without claiming
that unchanged configuration proves unchanged residency. Final verification and
full157 remain deferred until terminal optimization.

## Performance-only result

| Trial | Init seconds | Admission commits* | Max transaction objects | Max transaction bytes | Peak RSS bytes | Database bytes |
|---|---:|---:|---:|---:|---:|---:|
| 015-coalescing-first | 3.952135 | 137 | 6144 | 4192561 | 95076352 | 551010304 |
| 016-filter-control-second | 4.808933 | 1158 | 512 | 524288 | 82886656 | 550817792 |
| 017-reverse-filter-first | 4.802288 | 1155 | 512 | 524286 | 83607552 | 550924288 |
| 018-reverse-coalescing-second | 3.949917 | 138 | 6019 | 4185785 | 98189312 | 550961152 |

Mean Init elapsed: coalescing 3.951026s versus
filter-only 4.805611s, 17.78% lower. Both run orders favor
the treatment. Candidate timings differ by about2ms; no statistical significance
or final qualification is claimed from these exploratory observations.

*The existing public admission receipt excludes the final root commit. Metrics
now count actual early SQL commits, not physical insertions; final transaction
size still contributes to maxima. Physical batch peaks remain512objects/512KiB.
The byte ceiling binds before the8191-object ceiling on this fixture.

Peak RSS rises by12.2MB and14.6MB in the corresponding pairs. This is a measured
tradeoff, not an assertion of unchanged memory. Closed database size changes by
192512bytes and36864bytes; full157 storage preservation remains unverified.
No imported library source, version or feature changed. Four-KiB pages and
existing encoders/physical grouping remain in place.

Existing release host build passed. Independent read-only review found no
blocking final-root atomicity or rollback issue. No tests, independent verifier,
or full157 ran. Any added regression test is deferred and is not a passing check.
The provisional implementation is retained for further optimization/terminal
qualification; no merge, push or published-release claim follows from this run.

Exact measured product patch and binary identity are archived in
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91-runs/r19-fast/build-015-coalescing/`.
Each trial above retains command, raw diagnostic/performance output, binary
identity and its closed Store under the same r19-fast directory.

Deferred regression added after the measured product snapshot, with no subsequent
product changes: `objects::tests::init_cohorts_bound_sql_commits_flush_empty_final_and_rollback_pending`
in objects.rs. It covers cohort ceilings, pending/prior rollback and empty-final
publication. It has not been compiled or run. Run it during terminal verification.

Diagnostic early-commit time fell from1.906/1.897s to1.372/1.371s in the paired
observations. This supports reduced publication cost; it does not by itself
attribute the entire public elapsed improvement to commits.
