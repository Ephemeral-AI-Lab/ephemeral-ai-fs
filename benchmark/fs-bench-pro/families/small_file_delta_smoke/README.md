# Small file delta smoke

One exploratory case: `small-file-delta-10x30-v1`. Ten files, thirty ordinary full-file Exec/FUSE saves and Created commits. Native Init; reopen the same measured Store to verify all 31 retained states after allocation is frozen. Fixed watchdogs: 600 seconds per phase, 30 seconds per operation. No numerical performance gate.

See `docs/roadmap/0.1/0.1.5/delta-encoding-benchmarks.md`.
