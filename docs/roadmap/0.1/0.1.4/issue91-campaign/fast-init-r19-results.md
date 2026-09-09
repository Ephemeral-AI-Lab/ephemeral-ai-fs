# R19 fast Init observations (provisional)

Performance-only observations on the existing host namespace100000 Init route.
Verification and full157 remain deferred by user instruction. These are not final
qualification results. Imported dependency sources, versions and features are
unchanged. SQLite pages remain 4096 bytes.

| Trial | Init seconds | Peak RSS bytes | Closed database bytes |
|---|---:|---:|---:|
| 001-g8-control | 4.997661 | 78364672 | 550887424 |
| 002-direct-assembly | 5.489487 | 78086144 | 550928384 |
| 003-borrowed-locators | 5.046067 | 79790080 | 550887424 |
| 004-fresh-negative-filter | 5.091723 | 84803584 | 550993920 |
| 005-file-growth-chunk | 5.390952 | 77430784 | 550961152 |
| 006-incremental-blob | 5.294474 | 77348864 | 550834176 |
| 007-tiny-full-group | 5.252232 | 82362368 | 549724160 |
| 009-paired-tiny-first | 5.622155 | 79413248 | 549773312 |
| 010-paired-control-second | 5.543671 | 79970304 | 550965248 |
| 011-paired-filter-first | 4.604688 | 84836352 | 550961152 |
| 012-paired-control-second | 5.223219 | 81281024 | 550932480 |
| 013-reverse-control-first | 5.172076 | 79921152 | 550883328 |
| 014-reverse-filter-second | 4.430620 | 86638592 | 550875136 |

The first filter observation (004) did not beat the earliest control. Subsequent
adjacent pairs favor the filter in both execution orders: 011/012 by 11.84%, and
014/013 by 14.34%. Mean candidate/control elapsed is 4.518/5.198 seconds, about
13.09% lower. Retain the unfavorable first observation; host/cache conditions are
uncontrolled and these pairs are exploratory, not statistical qualification.

The filter skips SQL membership lookup only for definite negatives in a Store
that starts empty under the operation permit. Possible positives retain exact
lookup/authentication. Publication updates the bitmap before commit and epoch
release; failed commits can only add false positives. The 4 MiB bitmap reduces
the existing seen-index allowance from 16 to 12 MiB. Physical batch and transaction
limits stay 512 objects/512 KiB, and existing encoding/dependency paths remain.
Two additive nonce-scoped counters expose skipped and queried probe IDs. Trial011
skipped422052 IDs and queried2104 IDs.

Actual peak RSS increased by 3.6–6.7 MB in the paired observations despite the
allowance repartition. Terminal resource qualification must measure this honestly;
the unchanged allowance is not evidence of unchanged resident memory. The paired
closed database sizes are within 78 KiB and do not establish full157 storage parity.

The active filter implementation is provisional. Terminal checks must cover empty
and nonempty Stores, duplicates/collisions, publication epochs, failure rollback,
final root/storage behavior, and memory. No such checks ran in this fast lane.
Earlier R15 checks are not proof of this filter.

Source patches, exact commands, binary identities and raw output are retained at
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91-runs/r19-fast/` under each trial;
the reusable filter binary and identity are in `build-011-filter-pair/`.
Other measured treatments are set aside. The user subsequently approved bounded transaction coalescing; see the R23
experiment declaration for the changed transaction boundary.
