# Stage 1 evidence pointer — #111 Commit inner attribution

Contract: [commit-stage1-contract.md](commit-stage1-contract.md).

Evidence root (outside the repository, all attempts retained):

```text
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage1-evidence/20260911T163220Z
```

Contents at freeze time (grows during collection; never edited afterwards):

- `custody/before-*` — preexisting workspace state recorded before any work:
  HEAD, `git status`, the byte-exact tracked dirty patch
  (`before-tracked-binary.patch`, sha256
  `44fdf679f24deead8e5f0d7dc8b2d0dd3a06346d56d8477058ebc69db0f6c2cc`) and the
  sha256 of every preexisting untracked compaction-removal / issue112 / issue113
  file.
- `custody/seals.py` — the source/product/workload seal computation used for
  drift checks.
- `stage1-plain/`, `stage1-diagnostic/` — isolated git worktrees at Stage 1 HEAD
  with the declared dirty treatment applied byte-exactly.
- `build-host-plain.log`, `build-image-plain.log` — plain-arm build stdout/stderr
  and exit codes.

Reproduced identities (drift check, both arms before instrumentation):

```text
product  seal 760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932
plain    source seal 490938083f8a7d7ecec166ae2e20c7287d0f7c1e890c504fd5c5c05217be3a8e
workload sha256 c6f1e4b15fce502ee1c08bd875e758beb099d3398394831faeca507c4b4e579b
```

The current dirty worktree without the harness hashes to source seal
`371d5dc40336492ed4aa969f4d210536bfbf740a8e0ac0f8a7c44e1ada1ac38b` and the same
product seal — exactly the retained-main build recorded by the completed
baseline — so the Stage 1 treatment is byte-identical to the measured baseline
treatment. Only the documentation commits `faaa03933` and `e5ea7ffca` differ;
neither is inside the sealed input set.

Archived evidence roots, including
`layerfs-commit-baseline-evidence/20260911T142606Z` and the #104/#109/#110
roots, stay read-only. No archived collector, validator, analyzer or finalizer
is executed in place.
