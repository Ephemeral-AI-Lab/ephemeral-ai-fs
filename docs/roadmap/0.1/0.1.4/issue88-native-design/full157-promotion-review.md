# Independent SP promotion review

**Recommendation: permit exactly one frozen full157 control/candidate pair as bounded research continuation, with no parameter change. This is not release or performance acceptance.** The completed smoke and read gates support that next applicability test; they do not establish the 159,163,199-byte component milestone, 134,221,004-byte allocation target, or generally acceptable PREFIX-chain read cost.

## Evidence checked

Read campaign `issue88-SP-public-read-1`: summary PASS, 24/24 rows, both original-source-unchanged flags true. Independently read all 24 result JSONs and checked read/digest correctness, daemon/zero-Docker execution receipt fields, phase time limits, resource PASS, present end-session FUSE receipts, no Commit, successful Clean/head checks and owned-container removal. Utility identities match across all rows. Rehashed only summary, identity and the 24 small result JSONs against the existing manifest; no Store or original evidence tree was scanned. Full producer custody revalidation remains the parent’s responsibility.

## Public Exec read-and-count costs

Each arm has three observations per cell; ns are exact. These include shell/dd/wc, daemon execution and output drain, not isolated decoder latency. No warm-up or valid observation is discarded.

| Case / operation | Control raw ns (repetition order) | Candidate raw ns | Control median ns | Candidate median ns | Median difference ns | Difference |
|---|---|---|---:|---:|---:|---:|
| sdk-text-32k / range | [8049541, 4612250, 4482875] | [74423459, 4389750, 5618083] | 4612250 | 5618083 | 1005833 | 21.808% |
| sdk-text-32k / full | [5508333, 4753666, 4665417] | [6263292, 6865167, 5187042] | 4753666 | 6263292 | 1509626 | 31.757% |
| sdk-binary-8m / range | [4437583, 5865000, 6943167] | [6781625, 6034125, 5995458] | 5865000 | 6034125 | 169125 | 2.884% |
| sdk-binary-8m / full | [62568666, 58624375, 58773416] | [65213709, 64999875, 61394916] | 58773416 | 64999875 | 6226459 | 10.594% |

The text full median exceeds roughly 30% (+31.757%) but adds 1,509,626 ns. The owner’s approximate foreground-cost guidance calls for absolute-time judgment here, not an automatic percentage pass or waiver. The binary full median adds 6,226,459 ns. These regressions remain part of the result.

The first candidate text-range observation is 74,423,459 ns versus the first control observation 8,049,541 ns. Keep both. Candidate host CPU is 2,086,792 ns and native decode time is 6,917 ns; its execution receipt assigns 26,593,542 ns to spawn, 5,315,375 ns to supervisor queue and 40,669,833 ns to runtime (daemon nested timing also elevated). Those nested fields must not be added again to public elapsed. The receipt localizes where the wall interval is reported; it does not establish a cause such as cold cache, scheduler noise, codec cost or infrastructure failure. No sample is removed or retried on that inference.

| Case / operation | Control median host CPU ns | Candidate median host CPU ns |
|---|---:|---:|
| sdk-text-32k / range | 1303542 | 1568542 |
| sdk-text-32k / full | 1225916 | 1694291 |
| sdk-binary-8m / range | 1687082 | 1997500 |
| sdk-binary-8m / full | 28407417 | 29510708 |

Every candidate timed range reads one native FULL record (4,096 raw bytes, depth0); every full-file operation decodes three native FULL records (12,288 raw bytes, all depth0). Native dependency-edge counts are zero throughout this read campaign. The 8,388,608-byte full-file observation therefore is **not** an all-native eight-MiB read or a PREFIX-chain experiment. Its remaining reads use the retained legacy representation. Native decoding takes 3,666–18,959 ns across these rows. That timing alone cannot explain or dismiss the public latency changes, since it excludes all other path work.

FUSE per-phase counters correctly remain null. Measured end-session counters span setup+read+digest+end: range rows report 8,192 bytes/one request, text-full 32,768/one, binary-full 8,388,608/64. They are not separate read versus digest denominators. Write counters remain unavailable; functional nonmutation is supported by fixed commands, count/digest checks, Clean end, unchanged head and no Commit. Fresh application contexts do not imply cold OS caches.

## Smoke evidence and promotion limits

Read the six arm/family performance and verification summary sets: every case reports performance PASS and cleanup PASS, and all six verification summaries PASS. Parent reports 33 verified mappings / 34 raw acknowledgement rows per arm; the small-files read-only row is not an additional retained mapping. No failing baseline Store regression is waived by these public successes.

The deepseek-five final acknowledgement changes logical database length 2,748,416→2,719,744 B (-28,672), while allocated Store changes 3,153,920→3,194,880 B (+40,960). Canonical populations differ by one object/108 B; equal filesystem oracle is the matching correctness dimension, not presumed identical canonical identity. Its summed public Commit is 355,518,501→223,631,459 ns and Exec 1,048,249,707→971,633,374 ns. These small-history results are encouraging applicability evidence, not a full-history savings forecast. Its normal public Exec receipts actually include native PREFIX depth1–3 work, unlike the fixed read campaign.

Do not hide neutral or negative buckets: small-files logical/allocated grows 200,704→204,800 B with a different canonical population; FUSE-binary logical grows 8,601,600→8,605,696 B. SDK-binary logical is unchanged at 8,630,272 B while allocation falls 9,654,272→8,630,272 B; that allocation-only change is not proof of encoded-content savings. SDK-text and FUSE-text final logical sizes are unchanged.

The next pair should retain the exact C+S1 control and C+S1+P replacement package, frozen workload/oracles/environment, existing resource/deadline/cleanup stops, source/binary/image seals and all negative observations. Run once per arm under the shared lock. Preserve acknowledgement allocation as primary and take the declared single pre-verification logical snapshot; use the validated additive native census for exact physical/base/role accounting and full157 normal historical verification. Abort promotion on an integrity, resource, identity or cleanup failure. Do not raise limits, change grouping/base selection, collect replacement samples or weaken tests after results.

This is a finite cost decision: proceed to learn the public retained-history result despite modest observed read regressions and one unexplained 74.4-ms outlier. The short fixed read probe cannot guarantee chain-heavy performance. Full-pair results must report separate Commit and Exec costs, native depth/dependency work and storage decomposition before any adoption decision; no invented storage gate or general 30% exemption is granted.
