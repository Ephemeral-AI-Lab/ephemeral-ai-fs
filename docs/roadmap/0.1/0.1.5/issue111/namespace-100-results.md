# Registered namespace100 Init run (#111)

Requested case: **init_namespace / namespace-100-compact-v3**, seed1, n1. **Performance and independent verification PASS.** Original registered pseudorandom fixture:100 files,1 data directory,5,000,000 logical bytes (1 empty,78 tiny,15 small,5 medium and1 anchor file). Public operation Client::initialize_layerstack; unchanged promoted-uncompacted product.

| Metric | Result |
|---|---:|
| layerstack_init_ns |21,637,708 (21.638ms)|
| Original logical bytes |5,000,000|
| Canonical objects |365|
| Canonical bytes |5,029,235|
| Store apparent bytes |5,160,960|
| Store allocated bytes |5,160,960|
| initialization_disk_read_bytes |0|
| initialization_disk_write_bytes |122,880|
| User CPU ns |17,034,666|
| System CPU ns |16,770,917|
| Process peak RSS bytes |30,932,992|
| Process swaps |0|

n=1; median=min=max21,637,708ns. No paired comparator or speedup claim. This is **cache-resident**, not cold: the runner emitted fixture_cache_profile=reused-first-sample-uncontrolled but observed0 disk-read bytes. The2.7s cold namespace100000 gate is not applicable to this smaller case. Store allocation is160,960B (3.2192%) above logical content, consistent with the incompressible fixture and representation/index/page overhead. No compression setting or product code was changed.

Independent verification PASS: import counts100 files/5,000,000B; persisted root reopen equality;6 sampled files/148,294B read-verified through FUSE; cleanup PASS. This is the registered import-counts-reopen-and-sampled-fuse-v1 proof, not exhaustive whole-file/namespace verification. Verifier wall1.593s, within45s work/59s hard bounds.

Host binary SHA256:`51fb9e01e8990746a1ba1624e46e64446bc84a6451c35c851a3fbded21392f72`. Product seal:`95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4`. Source seal:`216f31304407442e18b6f32672099ac1a2590f245681834290deccb9cb581133`. Matching image:`sha256:a44aa289897762f2e98c358d432e3a7ad0bc300f2a231b5370c166c101b6d02a` (`layerfs-bench-infra:216f31304407442e`). Fixture digest:`c3ff9877a7edce88076b87d45b7da9ab7730836f10b4aff937f7b0cc82986ec1`. Host rebuilt/requalified through shared/runner.py; matching Linux image rebuilt after the previously listed tag could not be inspected. Native operation/Store on macOS; Docker used for standard daemon/FUSE proof. Runner measurement lock used throughout. No compaction/repack/VACUUM/GC or source edit during qualification/execution.

The first verifier invocation was rejected before execution because exact source/input arguments were missing. That attempt is retained; one corrected proof invocation supplied both identities directly from the performance header. The valid performance sample was not rerun.

Before the owner's clarification, one assumed real-source100 subset sample had started and failed its separate cold-cache validation. It was stopped, retained separately, and never relabelled as this registered case. See the status note in real-source-100-contract.md.

Evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-namespace100-evidence/run-20260911T002827Z/registered-namespace100`. Exact commands/logs/exits: performance.command.json, performance.log, verification.command.json/log/exit.json (argument rejection), verification-qualified.command.json/log/exit.json (PASS). Raw performance/perf.jsonl and verification-qualified/verification.json; summary.json; protocol.md; manifest.json. Existing original compaction-removal dirty work remains preserved. No release/tag/deployment.
