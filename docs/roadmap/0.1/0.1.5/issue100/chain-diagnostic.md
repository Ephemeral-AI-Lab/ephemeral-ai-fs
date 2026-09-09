# Actual-file chain diagnostic

Prospective policy: [bounded predecessor amendment](bounded-predecessor-amendment.md),
committed as `539825b19` before encoding. Original selected fixture and Git repository
are immutable. The ignored Rust diagnostic uses the product's pinned static codec,
authenticates reconstructed canonical identities and asserts original bytes.

| Family | Existing FULL-anchor retained record + directory bytes | Bounded immediate predecessor | Net saving |
|---|---:|---:|---:|
| translation-prompt-v4/request-response.expected.json | 110,601 | 103,848 | 6,753 |
| docs/config-catalog.zh.md | 39,760 | 39,760 | 0 |
| ui-primitives/src/icons/index.tsx | 90,137 | 80,109 | 10,028 |

Every distinct FULL and DELTA is counted once, including physical bases. Icons
FULL bytes increase from 26,614 to 65,791 when the decoded-closure ceiling forces
refresh; DELTA bytes fall from 63,523 to 14,318. This is a net record saving, not
credit for removing retained bases. The 133,273-byte CDC predecessor remains
ineligible, and its 129,991-byte successor remains FULL in this policy.

These are three-family diagnostic costs, **not allocated Store measurements**.
CDC storage, pack headers, metadata, SQL/index/page overhead and offline oracle
storage are excluded from this table; they must be counted in the public smoke.
Debug-build encode/decode nanoseconds are retained in the log and are not public
save/Commit performance. Icons total diagnostic decode time increased from
52,832,623 to 79,712,835 ns; this motivates measuring historical-read cost.

Raw command/log: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/diagnostic-1*`.
The focused depth/closure/integrity/exact-CAS and schema8 nonpromoting/upgrade
checks also passed; the old upper-range exact-CAS check passed because this
candidate changes its reader. The final acquisition-budget guard prompted a
focused recheck, not a new diagnostic encoding. No broad test/Clippy/doctest ran.

The candidate warrants one complete ten-state smoke to quantify storage versus
read amplification. This diagnostic does not establish that eight edges are
necessary or that 45 million allocated bytes is achievable.
