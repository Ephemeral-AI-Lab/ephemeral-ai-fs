# Issue100 fast iteration: every third snapshot

Frozen 2026-09-10 from the owner's request to reduce commit work during iteration.

## Selection and identity

- Scenario: `deepseek-stride3-v1`; harness case: `deepseek-stride3`.
- Original manifest SHA256: `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`.
- Pinned source tip: `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`.
- Select exactly original indices **1, 4, 7, …, 157**: `range(1, 158, 3)`.
- Exactly **53** selected states, in original order, keeping the first and last state. Record consecutive campaign index1–53 and each original `full157_index`.
- Preserve each selected source SHA, tree ID, manifest hash, oracle hash and logical byte count. Derive direct transitions between successive selected states; do not replay or commit skipped states.
- Use the existing selection/transition fixture machinery; retain the original full157 and ten-state profiles without changing their selection or evidence.

## Measurement boundary

This is the default fast development track for new optimization candidates. Its purpose is to reject poor designs before a full157 campaign. It is **exploratory**, with **release admission false**.

Commit count falls from157 to53 (66.24% fewer). This is not a measured threefold wall-time improvement: selected transitions may contain more changes, while setup, preparation, encoding, verification and cleanup have their own costs. Preserve actual phase timings.

Every comparison needs a fresh candidate Store and an applicable matched LayerFS control using these exact53 states. Before reporting a Git ratio, create and measure a separate Git53 control with exactly these selected trees/commits and the established Git storage measurement policy. **Git53 is not measured yet.** Do not reuse the38.22MB ten-state or56.37MB full157 Git size as its baseline.

Skipping verification steps against an existing157-commit Store does not create a53-commit storage result. Similarly, filtering an existing157-object graph without rebuilding its retained dependencies is not a valid53-state experiment.

## Execution and validation

1. Prepare/authenticate the fixed53-state fixture, its direct transitions and original oracles. Record fixture and contract hashes.
2. Run the candidate/control through the existing real save/Commit session for exactly53 selected states. Freeze census/allocation before same-Store historical verification and cleanup.
3. Require the existing byte/type/mode/symlink/link semantics, exact original-state oracles and clean teardown. Preserve all original inputs and previous output directories.
4. Report selected indices, Created outcomes, allocated/logical storage, complete dependency/index costs, Commit timing sum/median and setup/verification/cleanup scopes.
5. Preserve negative outcomes and fixed-policy evidence in the optimization ledger. Advance only promising candidates to all157 states.

Full157 remains the final history/scale validation track. The already-running structural full157 verification is allowed to finish and keeps its own result. The53-state profile does not retroactively change prior measurements or product correctness obligations.
