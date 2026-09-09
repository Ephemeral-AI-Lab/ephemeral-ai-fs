# Chain-1 complete FULL-population diagnosis

Read-only ten-state diagnosis after correcting the premature full157 continuation. This is the verified chain-1 Store, not full157. No encoding, product changes, builds, benchmarks, or parameter sweeps were performed.

The remaining SmallContent FULL population reconciles exactly to **14,664 selected objects / 30,497,635 compressed frame bytes**. Its dominant category is a path absent from the preceding retained snapshot. This proves the immediate same-path predecessor is absent for that logical path; it does **not** prove FULL encoding unavoidable or exclude an available similar object at another path. Logical predecessor observations do not establish which hints the importer delivered.

| Logical first-appearance category | Objects | FULL frame bytes |
| --- | ---: | ---: |
| initial_snapshot_no_prior_snapshot | 259 | 473,253 |
| changed_path_previous_Small | 31 | 310,894 |
| later_snapshot_new_path | 14,368 | 29,601,395 |
| CDC_crossing | 6 | 112,093 |
| Recurring/CAS or mixed-path ambiguity | 0 | 0 |
| **Total** | **14,664** | **30,497,635** |

A canonical Git blob appearing at multiple paths is counted once per selected SmallContent object. Classification considers every path at its first appearance; mixed predecessor facts would be reported separately. Physical pack insertion steps match first appearance for this population. The 259 initial-snapshot objects lack earlier retained snapshots, but may still share content within their snapshot. No portion is labelled mathematically unavoidable.

| First smoke step | Objects | FULL frame bytes |
| --- | ---: | ---: |
| 1 | 259 | 473,253 |
| 2 | 1,172 | 2,673,245 |
| 3 | 976 | 1,957,228 |
| 4 | 1,496 | 2,829,400 |
| 5 | 1,731 | 3,139,309 |
| 6 | 829 | 1,623,870 |
| 7 | 1,553 | 3,874,742 |
| 8 | 1,881 | 4,155,732 |
| 9 | 2,131 | 4,041,854 |
| 10 | 2,636 | 5,729,002 |

The original saved Git pack graph partitions exactly the same FULL frame population:

| Git representation / whole base-chain availability | Objects | LayerFS FULL frame bytes |
| --- | ---: | ---: |
| Git_FULL | 5,033 | 11,271,235 |
| Git_DELTA_future_closure | 7,858 | 16,583,648 |
| Git_DELTA_same_snapshot_closure | 987 | 1,101,167 |
| Git_DELTA_earlier_closure | 786 | 1,541,585 |

**1,759 FULL objects / 2,431,054 frame bytes** have an actual Git base with complete closure available by the target snapshot and with disjoint first-appearance paths. This is a concrete different-path candidate cohort, not a saving estimate: Git representation, base eligibility, ordering within the snapshot, decoded closure and LayerFS encoding cost remain unproven. Git entry sizes exclude the cost of establishing alternate base representations. The larger future-closure category does not exclude an earlier similar candidate; the saved Git choice alone cannot answer that question.

Examples suitable for the next bounded candidate investigation:

| Target | First step | FULL frame B | Actual Git base / first step / raw B |
| --- | ---: | ---: | --- |
| `.agents/notes/archived/architecture/2026-08-11-repository-naming-contract-and-rename-ledger.zh.md` | 10 | 21,434 | `.agents/notes/proposed/architecture/2026-08-11-repository-naming-contract-and-rename-ledger.zh.md` / 8 / 56,620 |
| `.agents/notes/archived/architecture/2026-08-11-repository-naming-contract-and-rename-ledger.md` | 10 | 18,891 | `.agents/notes/proposed/architecture/2026-08-11-repository-naming-contract-and-rename-ledger.md` / 8 / 58,630 |
| `.agents/notes/archived/architecture/2026-07-19-gui-layering-and-rpc-protocol.zh.md` | 10 | 13,094 | `.agents/notes/implemented/architecture/2026-07-19-gui-layering-and-rpc-protocol.zh.md` / 8 / 25,831 |
| `snapshots/session/cordis-inspect-jsdoc/tool-schemas.expected.json` | 9 | 13,052 | `snapshots/session/agent-instructions/tool-schemas.expected.json` / 9 / 76,195 |
| `snapshots/session/code-mode-read-image/system-prompt.expected.md` | 9 | 12,342 | `snapshots/session/cordis-inspect-jsdoc/system-prompt.expected.md` / 9 / 54,305 |
| `snapshots/session/code-mode-turn/system-prompt.expected.md` | 9 | 12,335 | `snapshots/session/code-mode-read-image/system-prompt.expected.md` / 9 / 37,213 |

Move families in `.agents/notes/{proposed,implemented,archived}/` and `snapshots/session/*/{tool-schemas.expected.json,system-prompt.expected.md}` have concrete prior or same-step different-path witnesses. They deserve bounded provenance/order studies. Same-snapshot availability is not proof of prior admission. The existing 128-entry signature-ring diagnostic is a narrow coverage result, not a population-wide impossibility result.

Basename aggregation over this FULL population (each selected object attributed to one representative path):
- `index.ts`: 475 objects / 1,290,160 frame bytes.
- `README.zh.md`: 455 objects / 1,273,390 frame bytes.
- `README.md`: 488 objects / 1,022,652 frame bytes.
- `session.jsonl`: 257 objects / 568,742 frame bytes.
- `tool-schemas.expected.json`: 41 objects / 305,399 frame bytes.
- `session.v2.jsonl`: 129 objects / 247,928 frame bytes.
- `package.json`: 422 objects / 218,826 frame bytes.
- `invariant.ts`: 324 objects / 215,898 frame bytes.
- `system-prompt.expected.md`: 46 objects / 197,763 frame bytes.

Repeated basenames alone are not similarity evidence and do not authorize a global filename index. Largest concrete individual paths include the translation request-response family (92,550 B across two FULLs), two provider-page PNGs (71,485 / 66,355 B), icon index (65,741 B across two FULLs), and config catalog Chinese (31,678 B). The JSON retains all 14,664 rows, logical priors, Git direct bases and closure steps, plus ranked paths.

Custody and reproduction:

- Store: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/chain-1/deepseek-ten/host-runtime/store.sqlite`.
- SHA-256 before **and** after: `46667cb134675054c20fe9e6ba4624b4ce3f96f0f8d7947e763911a6094829f5`; inode 780433098 unchanged.
- Existing `layerfs-infra-measurement.lock` held for the read; SQLite `mode=ro&immutable=1`. Only selected FULL payloads were decompressed, using the existing ctypes Zstandard library; no DELTA reconstruction or compression.
- Every prepared `manifest.tsv` hash was checked against the frozen fixture seal. Each decompressed raw FULL was mapped to an existing original-fixture Git blob by its Git SHA-1. This diagnostic is not a substitute for the already completed authenticated same-Store verification.
- The original saved `git-pack-attribution-raw.txt` supplied the Git graph. The frozen census supplied expected FULL object/frame totals. Store verification lifecycle changes were not confused with the preverification census digest.

```sh
python3 /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/full-population-diagnosis.py
```

The script creates its JSON exclusively and refuses to overwrite existing evidence. Existing artifacts:
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/full-population-diagnosis.py`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/full-population-diagnosis.json`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/full-population-diagnosis.log`

This remains frame attribution. It does not alter the 56,668,160-byte measured allocation, achieve 45 MB, establish a storage lower bound, or justify final full157 work before the short-loop objective is properly resolved.
