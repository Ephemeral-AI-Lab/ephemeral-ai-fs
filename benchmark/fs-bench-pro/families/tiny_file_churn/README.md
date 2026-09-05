# Tiny-file churn benchmark

Family ID: `tiny_file_churn`.

The approved [mixed bulk profile v3](../../../../docs/roadmap/0.1/0.1.3/tiny-file-churn-mixed-v3.md) changes only bulk create/delete tiers 100 and 500:

| Tier | Affected files | Total affected content | Distribution |
|---|---:|---:|---|
| 100 | 1,000 | 100 MiB | 1 × 50 MiB; 800 × 4 KiB; 199 medium files sharing the remainder |
| 500 | 5,000 | 500 MiB | 3 × 100 MiB; 4,000 × 4 KiB; 997 medium files sharing the remainder |

The separate 200-file / 1 MiB witness remains unchanged. New bulk IDs end in `-mixed-v3`; low-tier compact and individual-operation cases are unchanged. The linked specification defines exact bytes, ordering, identity, migration and verification.

**Implemented:** specification `66983181a`, migration `0ac1ebcf4`. The active registry contains the four new mixed-v3 IDs. The original unversioned high-tier bulk receipts remain historical and are not a product-speedup comparator.

Use the existing host-owned SQLite runner and [QUICKSTART](../../QUICKSTART.md). Registration, generation, oracle/sampling, input/cache identity and assessments are migrated together. Select one revised tier-100 case at a time; tier-500 performance remains deferred. No product case/size optimization switches.
