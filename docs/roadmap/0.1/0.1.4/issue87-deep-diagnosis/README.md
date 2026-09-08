# Deeper issue87 diagnosis

**TARGET IDENTIFIED: shared correspondence coverage saturates.** Its useful
compressed savings remain unmeasured; one unchanged-policy coverage diagnostic
is still the next step. This additive investigation uses existing sealed
metadata/receipts plus small synthetic exact-code probes, without replay.

- [Reconciled findings and narrowed next step](findings.md)
- [Correspondence source and actual-cursor proof](correspondence.md)
- [Admission/matcher independent diagnosis](admission.md)
- [Independent proof and diagnostic-schema review](independent-review.md)
- [Sealed byte bounds](published/coverage-byte-bounds.json)
- [All158 checkpoint bounds](published/coverage-byte-bounds.csv)
- [Derived pack admission checkpoints](published/derived-pack-checkpoints.csv)
- [New additive manifest](published/manifest.sha256.json)

The prior immutable report is not rewritten. This addendum corrects its overly
broad uncertainty about admission chronology and nohint byte magnitude:
per-checkpoint retained-prefix and admission cardinalities establish selected
admission checkpoints; nohint canonical bytes are585,473,955–598,419,082.
That interval does not forecast compressed savings.

New external artifacts:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue87-deep-diagnosis-bd220ae-1`.
The old analysis metadata index is accessed immutable/read-only; no original
Store access, second payload census, product edits, benchmark changes,
retained-data encoding or PR merge. The tiny synthetic matcher/cursor probes
are algorithm witnesses, not frozen workload samples.

Reproduction (from repository root):

```sh
python3 docs/roadmap/0.1/0.1.4/issue87-deep-diagnosis/coverage_bounds.py --self-test
python3 docs/roadmap/0.1/0.1.4/issue87-deep-diagnosis/review-check.py
python3 docs/roadmap/0.1/0.1.4/issue87-deep-diagnosis/correspondence-receipts.py
```

`coverage_bounds.py SEALED_OLD_ANALYSIS NEW_OUTPUT` runs metadata-only derivation;
NEW_OUTPUT must not exist. The two probe directories use the existing local
product libraries; their reports contain offline Cargo commands. Keep compilation
output outside the source tree and do not recursively format included product
modules. All recorded checks passed; no product optimization is implemented.
