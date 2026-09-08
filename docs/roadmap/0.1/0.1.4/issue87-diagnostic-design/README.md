# Focused correspondence/handoff diagnostic design

The fuller source/evidence investigation and executable prototypes are complete.
No production instrumentation or replay is included.

- [Findings, new geometric bound, and prospective contract](findings-and-contract.md)
- [Exact cursor observation design and layout checks](correspondence-design.md)
- [Pre-CAS work ownership, grant lifecycle and pack provenance](admission-design.md)
- [Independent validation design](validation-design.md)
- [Aggregate validator](validate_diagnostic.py)
- [Sealed new no-overlap bound](published/no-overlap-bounds.json)
- [All157 per-checkpoint bounds](published/no-overlap-bounds.csv)
- [Artifact/source manifest](published/manifest.sha256.json)

Completed legitimate no-overlap can account for at most65,446,350 canonical bytes;
at least240,892,334 bytes of the predecessor-present/nohint cohort need another
explanation under the documented source/span premises. This is not a prediction
of useful DELTA or compressed savings.

The chosen prospective design observes existing reservation decisions, carries
two named diagnostic bytes only if actual layout/capacity equivalence is proven,
and conserves occurrence work separately from initially missing attempts and
selected winners. Reused-triggered work is not automatically avoidable. Successful
transaction pack ranges plus one final authenticated locator join provide bounded
provenance. No new global target index or per-Commit Store traversal is proposed.

Checks from repository root:

```sh
python3 docs/roadmap/0.1/0.1.4/issue87-diagnostic-design/no_overlap_bound.py --self-test
python3 docs/roadmap/0.1/0.1.4/issue87-diagnostic-design/admission-accounting-check.py
python3 docs/roadmap/0.1/0.1.4/issue87-diagnostic-design/validate_diagnostic.py --self-test
python3 docs/roadmap/0.1/0.1.4/issue87-diagnostic-design/correspondence/run.py
```

The cursor runner creates a disposable offline Cargo fixture using the existing
product cursor. It does not open a Store or encode workload candidates. The
validator checks aggregate metadata against externally authenticated provenance;
it is not a replacement for manifest/locator authentication. Its synthetic PASS
is not a measured full157 diagnostic result.

New external artifacts are retained at
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue87-diagnostic-design-363324b-1`.
Previous sealed directories remain immutable. Keep #87 open for the one narrowed
unchanged-policy diagnostic; no optimization, product/benchmark change or merge.
