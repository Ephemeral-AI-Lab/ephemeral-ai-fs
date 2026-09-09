# Git algorithm gap: findings and directions

Read-only study authorized by the owner, 2026-09-09, with three subagents covering
pinned Git 2.47.1 algorithms, LayerFS shared owners, and independent budget review.
No new benchmark, alternative encoding, repack, storage-policy change, build or
product modification was performed. Initial model-capacity errors were retried
successfully; they produced no benchmark attempts or product changes.

## Exact ten-snapshot gap

| Category | v0.1.5 | Git | Gap |
|---|---:|---:|---:|
| File-content packs / blob entries | 54,245,616 | 34,306,253 | 19,939,363 |
| Metadata/legacy packs / trees and commits | 6,559,114 | 1,566,186 | 4,992,928 |
| Remaining indexes, structures and allocation | 5,300,614 | 2,351,433 | 2,949,181 |
| **Allocated total** | **66,105,344** | **38,223,872** | **27,881,472** |

All buckets conserve measured allocation. Git's 32-byte pack envelope is in its
remaining bucket. LayerFS metadata includes five legacy chunk objects. Different
semantic structures and framing mean bucket differences are not pure codec costs.

Reaching 40,000,000 B needs 26,105,344 B net saved. A useful target budget is
34,306,253 B content + 2,500,000 B metadata + 3,193,747 B remaining allocation.
This is a budget, not a demonstrated or probabilistic prediction. Matching Git
content while keeping current metadata/other allocation would still cost
46,165,981 B. Both content and non-content savings are required unless content
compression outperforms Git enough to compensate.

## What Git actually does differently

Git groups candidates by type/name hash and descending size. Its pinned source
explicitly prefers larger bases because deleting data can be cheap; the base is
not restricted to a chronologically earlier version of the same path. The matcher
uses indexed byte matches and COPY/literal instructions, then zlib compression.
Candidate search, delta depth and data/index memory have limits; the pack itself
is not one giant shared compression stream.

In the measured pack, 18,604 of 23,597 blob DELTAs (78.84%) require a reconstruction
closure involving a later selected snapshot; the count is NOT necessarily direct
base first-appearance. Maximum blob depth is 23, tree depth 7. Later-dependent
blob DELTA entries occupy 4,519,458 B of 6,163,088 B total DELTA entries.
Reproducing that specific graph requires future access/deferred packing or replacing
older physical encodings. A different online graph might approach its bytes; the
present measurements do not establish that it can or cannot.

A depth-one reverse star can compress a growing history well: the final large
file is FULL, earlier shorter files are mostly COPY deltas. Thus depth one alone
is not the fundamental issue. LayerFS combines chronological admission, past-only
FULL anchors, fixed selected representations and no reclamation/repacking.

## Actual FULL-object correspondence

Root's bounded read-only decoder classified all 14,646 current SmallContent FULLs
by their Git physical representation, verifying raw Git blob identities and
reconciling 30,191,026 frame bytes to the frozen Store census:

| Git representation of our FULL targets | Objects | Our FULL frame bytes | Git target-entry bytes |
|---|---:|---:|---:|
| Also FULL | 5,025 | 11,177,274 | 10,605,176 |
| DELTA requiring later snapshot closure | 7,855 | 16,494,159 | 1,687,032 |
| DELTA with same-snapshot closure | 989 | 1,101,538 | 356,151 |
| DELTA with earlier closure | 777 | 1,418,055 | 265,491 |

The later-dependent population exposes a large graph difference. The earlier/same
populations have a 1,897,951-byte target-entry difference, useful for immediate
diagnosis but far below the complete target. Git entry costs exclude costs of
establishing alternate base representations; none of these differences is a net
online saving forecast. LayerFS admission's actual no-hint/ineligible/full-wins
fallback split is still not separately measured for SmallContent.

## A concrete online exclusion: crossing the cutoff

`scripts/snapshots/translation-prompt-v4/request-response.expected.json`:

- Smoke step 7 / full157 index 105: 133,273 B, CDC/extent in LayerFS. Git stores a
  FULL object of 49,944 packed bytes.
- Smoke step 8 / full157 index 122: 129,991 B, SmallContent in LayerFS. Git stores
  a direct depth-one DELTA against the preceding version, costing 5,492 B.
- LayerFS instead stores a FULL frame costing 50,626 B. Its SmallContent base
  eligibility requires an existing FULL SmallContent; the prior file is CDC-backed
  and exceeds the SmallContent raw bound.

This is a verified representation-boundary restriction, not a missing-similarity
claim, unbounded-history problem, or proof of codec inferiority. Merely changing
a pointer would violate the existing base grammar and bounds. A bounded logical
predecessor diagnostic is justified, but product support requires an explicit
physical-base/read-policy revision; the 128-KiB file-classification boundary need
not itself be swept or changed.

## Recommended investigation order

1. **Isolate graph restrictions from codec matching on a handful of real file
   families.** The identified cutoff-crossing example and one expensive growing
   history have concrete current/Git bases. If separately authorized, compare
   exact encoded targets with those bases, include every FULL/base record and
   reconstruction bound, and retain all outcomes. This is a diagnostic before
   another ten-state history, not a new synthetic benchmark family or sweep.
2. **Choose the foreground policy deliberately.** A bounded immediate-predecessor
   chain can avoid accumulated changes without future knowledge. A bounded
   already-available candidate can address some cross-file/no-hint cases. A
   bounded logical predecessor across the CDC/SmallContent boundary addresses
   the specific example. Each changes the original fixed base/depth/role policy;
   study authorization does not implement or qualify those revisions.
3. **Reduce metadata/index structure cost alongside content.** Git metadata is
   1.57 MB versus 6.56 MB here; locator pages alone are 4.09 MB in LayerFS. Keep
   required POSIX fields/identity/links and investigate compact representation
   and sharing at shared owners. The old metadata-anchor patch remains rejected;
   more metadata DELTAs is not evidence of lower allocation. No concrete metadata
   replacement has yet demonstrated the needed savings.
4. **Keep reverse repacking as a separately scoped alternative.** It is the most
   direct way to reproduce Git's actual future-base advantage, but requires safe
   physical-location replacement, dependency/read ownership and actual byte
   reclamation. Appending new encodings while retaining old packs cannot deliver
   that allocation benefit. The original task explicitly excludes this project;
   it is not silently authorized by a request to study Git.

Preferred first step is the small real-family diagnostic, followed by one explicitly
selected bounded online design. Do not promise 40 MB from short chains, stronger
compression, FULL refresh or copying Git's matcher alone. Allocation immediately
at Commit and allocation after optional repacking are different product claims;
any lifecycle revision must state and charge maintenance cost explicitly.

## Evidence and corrections

- [Pinned Git algorithm study](git-algorithm-study.md)
- [LayerFS code-owner study](layerfs-gap-study.md)
- [Independent accounting/budget review](git-gap-budget-review.md)
- [Machine-readable reconciliation](git-gap-summary.json)
- [Baseline contract and results](ten-snapshot-baselines.md)

Raw supplementary evidence lives in
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence/`:
`git-pack-attribution-raw.txt`, `git-pack-attribution.json`,
`small-full-git-attribution.json`, `online-candidate-example.json`.
`git-gap-summary.py` reproduces the Git availability calculation and allocation
reconciliation from retained raw output; `classify-small-full.py` records the
FULL classification method. No raw baseline receipt was changed.

The earlier full157 149–169 MB estimate was a scenario calculation with unmeasured
reduction assumptions, not a best-estimate forecast. Its percentages cannot be
transferred to this ten-state history. Likewise, the earlier 40 MB illustrative
budget with 10–15 MB FULL data was arbitrary: measured Git FULL blob entries
already cost 28.14 MB. The current whole-Store budget is tied to measured Git
categories and still requires validation before it becomes an expected result.
