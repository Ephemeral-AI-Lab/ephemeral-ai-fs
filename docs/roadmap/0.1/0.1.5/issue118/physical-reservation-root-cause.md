# Physical encoding reservation: root cause and current confirmation

The current `store-footprint-metadata-cardinality-100000` public workload
(100,000 files, 500,000,000 logical bytes) reproduced `physical encoding reservation`.
Its failed product command took 0.939 s; complete invocation took 32.950 s and included 31.394 s
of first-use deterministic fixture preparation. This was a failure, not a
completed latency sample.

The failure is in LayerFS's `PreparedAdmission::prepare_ordinary`, not SQLite,
rusqlite, zstd, or a 100,000-file format limit. The physical scratch guard compares:

```text
retained physical backing + live allocation associations
+ 1 MiB codec context + two bounded codec buffers <= 2 MiB
```

The captured calculation was 2,106,936 B versus 2,097,152 B, over by 9,784 B. This is a
conservative reservation calculation, not an observed process-heap peak.
Ownership tracing found two defects:

1. Every reserved prepared-object slot was 184 B, including an inline optional
   eight-word small-file signature even for native/metadata/ordinary records
   that do not use it. The failure logged 483 **capacity slots**; it did not log
   how many were occupied. Unused capacity still consumes memory.
2. The upstream-vector charge was computed before native processing and left
   unchanged after partitioning and dropping iterators. It could omit a new
   sibling vector at an intermediate flush, or charge a freed source vector at
   the final flush. Shrinking slots alone would not repair this ledger defect.

`a36c60891` puts signatures in one bounded per-admission vector and stores a
compact index in each prepared object. Actual slot size is 112 B. Producer
signature reuse survives; there are no per-object signature allocations or
rescans. Allocation is reserved before capacity is obtained, including old/new
buffer coexistence. `440584938` separately corrects upstream charges at every
lane/flush boundary. The 2 MiB physical and 6 MiB data limits stay unchanged.

The ledger proof observes 156,672 B at metadata intermediate processing,
147,456 B at metadata final/ordinary intermediate processing, and 0 upstream bytes at
the final ordinary flush. The current chunk's own capacity is charged separately.
The signature proof measured one near-limit cohort's reservation falling from
2,103,174 B under the old slot layout to2,072,862 B under the new layout, using actual
type sizes/capacities. That is not the identical earlier failure cohort and is
not a paired timing result.

Current confirmation on the combined source:

| Check | Result |
|---|---|
| Full existing 100,000-file input, debug Init | PASS; 21.34 s test / 26.799 s complete command after signature repair |
| Signature/CAS/failed allocation checks | PASS; unchanged signature capacity on rejected reservation |
| Actual intermediate/final lane accounting | PASS |
| Exact optimized public case after both fixes | PASS; 4.708209919 s product-call sum, 8.36 s complete command |
| Separate selected verifier | PASS; 7.52 s complete command, cleanup PASS |
| Swap/OOM and container cleanup | No observed container swap/OOM; cleanup PASS |

The selected verifier proves storage accounting, Store reopen and the registered
edit boundary (1,468 bytes on `d0000/f000000`); it explicitly does **not** claim
complete byte verification of all 100,000 files. Component authentication,
collision, rollback and bounded-memory checks complement that public proof.
The successful run retained 559,673,344 allocated Store bytes / 552,058,880 apparent
bytes; neither is a new comparative storage claim.

Executed host SHA256:
`fc9192f13c125c0663cd9e8bb53ae2e7361c7af61fccb11a7ffece9afdd89764`.
Image: `layerfs-bench-infra:2f6dbfc942093034`; source/harness/input identities are
in the raw receipts. Fresh evidence is under
`benchmark-results/host-store/issue118/20260912/`:
`cardinality-current-control/`, `cardinality-component-checks/`,
`cardinality-fixed-public/`, and `cardinality-fixed-proof/`.

All changes are in LayerFS-owned source/tests. No external library, system
library, Cargo registry source, dependency manifest or lockfile was patched.
This resolves the reproduced reservation failure at this source identity;
the umbrella's other optimization/capacity/terminal gates remain tracked in
`execution.md`.
