# Fixed Git-100 / Git-500 cohort

Timing tables display milliseconds; all target decisions use unrounded integer nanoseconds. Before observations are n=1. Final statistics use only the twelve explicitly named slots, with three repetitions per case and arm. Missing or invalid observations are not replaced.

**Scopes differ:** LayerFS is Create + required cold hydration + apply + six Git commands + LayerFS Commit + visibility + End. Native is apply + six Git commands only. Native is not a complete LayerFS lifecycle and receives no lifecycle target classification. Native raw admission_eligible remains false: these medians are matched scoped controls, not native product acceptance. MATCHED_COMPLETE describes cohort completion and identity matching, not release admission.

## Observations

| Run | Arm | Create | Exec | LayerFS Commit | Visibility | End | Total | Main gate |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| baseline-100 | LayerFS | 13.814 | 5765.452 | 35.206 | 0.071 | 12.733 | 5827.275 | TARGET_MISS |
| baseline-500 | LayerFS | 12.864 | 13977.050 | 103.938 | 0.084 | 24.370 | 14118.306 | TARGET_MISS |
| baseline-native-100-ownership-fixed | native | unavailable | unavailable | unavailable | unavailable | unavailable | 273.585 | N/A |
| baseline-native-500 | native | unavailable | unavailable | unavailable | unavailable | unavailable | 644.545 | N/A |
| final-100-layerfs-r1 | LayerFS | 12.015 | 1795.651 | 32.202 | 0.087 | 12.343 | 1852.297 | TARGET_MISS |
| final-100-layerfs-r2 | LayerFS | 10.196 | 1804.707 | 32.849 | 0.106 | 13.730 | 1861.588 | TARGET_MISS |
| final-100-layerfs-r3 | LayerFS | 15.423 | 1796.455 | 24.337 | 0.097 | 11.266 | 1847.577 | TARGET_MISS |
| final-100-native-r1 | native | unavailable | unavailable | unavailable | unavailable | unavailable | 267.418 | N/A |
| final-100-native-r2 | native | unavailable | unavailable | unavailable | unavailable | unavailable | 247.930 | N/A |
| final-100-native-r3 | native | unavailable | unavailable | unavailable | unavailable | unavailable | 249.184 | N/A |
| final-500-layerfs-r1 | LayerFS | 15.738 | 4594.074 | 81.109 | 0.073 | 21.035 | 4712.030 | TARGET_MISS |
| final-500-layerfs-r2 | LayerFS | 14.757 | 4489.401 | 75.152 | 0.087 | 21.816 | 4601.212 | TARGET_MISS |
| final-500-layerfs-r3 | LayerFS | 15.608 | 4525.094 | 73.333 | 0.093 | 21.387 | 4635.514 | TARGET_MISS |
| final-500-native-r1 | native | unavailable | unavailable | unavailable | unavailable | unavailable | 639.123 | N/A |
| final-500-native-r2 | native | unavailable | unavailable | unavailable | unavailable | unavailable | 634.505 | N/A |
| final-500-native-r3 | native | unavailable | unavailable | unavailable | unavailable | unavailable | 624.705 | N/A |

## Apply and all six Git commands

| Run | Apply | First status | Diff | Add | Cached check | Git commit | Final status | Workload |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| baseline-100 | 126.320 | 2163.521 | 3177.290 | 141.658 | 24.245 | 72.811 | 37.366 | 5761.906 |
| baseline-500 | 653.220 | 4934.302 | 7391.166 | 612.075 | 104.284 | 172.198 | 41.324 | 13971.978 |
| baseline-native-100-ownership-fixed | 101.376 | 38.433 | 101.336 | 13.163 | 2.540 | 7.971 | 5.430 | 273.585 |
| baseline-native-500 | 208.526 | 81.788 | 248.943 | 46.334 | 7.886 | 31.182 | 9.915 | 644.545 |
| final-100-layerfs-r1 | 102.839 | 523.281 | 967.199 | 91.928 | 13.961 | 38.223 | 34.591 | 1791.787 |
| final-100-layerfs-r2 | 104.721 | 493.999 | 995.504 | 100.367 | 14.103 | 36.535 | 33.939 | 1800.588 |
| final-100-layerfs-r3 | 93.976 | 480.362 | 1006.499 | 103.466 | 14.557 | 38.213 | 35.479 | 1792.698 |
| final-100-native-r1 | 100.663 | 36.349 | 99.184 | 12.312 | 2.580 | 8.069 | 4.926 | 267.418 |
| final-100-native-r2 | 77.167 | 39.355 | 100.071 | 12.797 | 2.466 | 7.940 | 5.041 | 247.930 |
| final-100-native-r3 | 80.632 | 37.276 | 98.718 | 13.363 | 2.580 | 8.447 | 5.285 | 249.184 |
| final-500-layerfs-r1 | 514.187 | 1112.724 | 2303.442 | 357.381 | 60.394 | 120.650 | 60.994 | 4590.567 |
| final-500-layerfs-r2 | 489.932 | 1107.198 | 2229.586 | 352.202 | 61.351 | 120.418 | 61.943 | 4485.630 |
| final-500-layerfs-r3 | 484.263 | 1117.781 | 2265.046 | 350.699 | 62.009 | 115.763 | 62.008 | 4521.402 |
| final-500-native-r1 | 218.772 | 78.502 | 244.182 | 43.060 | 7.719 | 31.257 | 9.121 | 639.123 |
| final-500-native-r2 | 188.339 | 86.201 | 257.766 | 45.793 | 8.407 | 30.791 | 8.675 | 634.505 |
| final-500-native-r3 | 205.902 | 88.443 | 236.265 | 41.453 | 6.897 | 29.936 | 9.090 | 624.705 |

## Final cohort median and maximum

| Case | Arm | n | Median ns | Maximum ns | Median ms | Maximum ms | Main | Stretch | Cohort |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| git-tool-100-mixed-v4 | LayerFS | 3 | 1852297167 | 1861588457 | 1852.297 | 1861.588 | TARGET_MISS | TARGET_MISS | MATCHED_COMPLETE |
| git-tool-100-mixed-v4 | native | 3 | 249183958 | 267417875 | 249.184 | 267.418 | NOT_APPLICABLE | NOT_APPLICABLE | MATCHED_COMPLETE |
| git-tool-500-mixed-v4 | LayerFS | 3 | 4635513874 | 4712029624 | 4635.514 | 4712.030 | TARGET_MISS | TARGET_MISS | MATCHED_COMPLETE |
| git-tool-500-mixed-v4 | native | 3 | 634505375 | 639123250 | 634.505 | 639.123 | NOT_APPLICABLE | NOT_APPLICABLE | MATCHED_COMPLETE |

Individual main-target misses: {"git-tool-100-mixed-v4": ["final-100-layerfs-r1", "final-100-layerfs-r2", "final-100-layerfs-r3"], "git-tool-500-mixed-v4": ["final-500-layerfs-r1", "final-500-layerfs-r2", "final-500-layerfs-r3"]}.

## Backing, canonical database and cache work

Database counters describe SnapshotReader queries and authenticated returned objects; they are not a count of all SQLite work. Nested receipt durations overlap and must not be added to lifecycle totals.

| Run | Backing calls | Request bytes | Backing wait ns | Host dispatch ns | DB calls | DB rows | DB bytes | Cache hits | Cache bytes | Local auth/read ns |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| baseline-100 | 10704 | 1889677 | 3675471516 | 1208680324 | 33973 | 35079 | 91970823 | 86808 | 9652922 | 1088540354 |
| baseline-500 | 24440 | 8471228 | 8870651396 | 2923788717 | 79098 | 82783 | 225451578 | 199664 | 21285341 | 2617667227 |
| final-100-layerfs-r1 | 784 | 1206574 | 529546339 | 263513059 | 11965 | 20720 | 20763049 | 57187 | 71089595 | 192723819 |
| final-100-layerfs-r2 | 784 | 1206574 | 498767336 | 243612121 | 11965 | 20720 | 20763049 | 57189 | 71095407 | 178411849 |
| final-100-layerfs-r3 | 784 | 1206574 | 488128164 | 241611181 | 11965 | 20720 | 20763049 | 57187 | 71089595 | 175519543 |
| final-500-layerfs-r1 | 1999 | 6909612 | 1183878281 | 568541875 | 24436 | 45003 | 48569790 | 149905 | 234247851 | 367299870 |
| final-500-layerfs-r2 | 1999 | 6909612 | 1165700908 | 558990440 | 24435 | 45003 | 48569790 | 149901 | 234236163 | 354507646 |
| final-500-layerfs-r3 | 1999 | 6909612 | 1129636988 | 521444507 | 24435 | 45003 | 48569790 | 149931 | 234323343 | 336255268 |

## FUSE callbacks and kernel reads

These are observed kernel/FUSE counters, not inferred from Git syscall totals. Fields absent from older baseline receipts are unavailable, not zero.

| Run | Lookup | Getattr | Open | Read | Read bytes | Flush | Release |
| --- | --- | --- | --- | --- | --- | --- | --- |
| baseline-100 | 14034 | 3962 | 12175 | 4744 | 28467200 | 12304 | 12304 |
| baseline-500 | 40290 | 8951 | 28535 | 10832 | 66740224 | 29144 | 29144 |
| final-100-layerfs-r1 | 9476 | 709 | 12175 | 89 | 1212416 | 150 | 12304 |
| final-100-layerfs-r2 | 9596 | 702 | 12175 | 89 | 1212416 | 150 | 12304 |
| final-100-layerfs-r3 | 9598 | 702 | 12175 | 89 | 1212416 | 150 | 12304 |
| final-500-layerfs-r1 | 28112 | 2144 | 28535 | 377 | 3440640 | 710 | 29144 |
| final-500-layerfs-r2 | 27952 | 2129 | 28535 | 377 | 3440640 | 710 | 29144 |
| final-500-layerfs-r3 | 27976 | 2144 | 28535 | 377 | 3440640 | 710 | 29144 |

## Kernel prefill and immutable acquisition

Prefill stores/bytes describe kernel page-cache delivery. Immutable fetches/cache hits and fetched bytes use the wired read_ahead fields; baseline unwired values remain unavailable. These counters have distinct scopes and must not be summed as unique payload traffic.

| Run | Prefill stores | Prefill bytes | Immutable fetches | Immutable cache hits | Fetched bytes |
| --- | --- | --- | --- | --- | --- |
| baseline-100 | unavailable | unavailable | unavailable | unavailable | unavailable |
| baseline-500 | unavailable | unavailable | unavailable | unavailable | unavailable |
| final-100-layerfs-r1 | 4655 | 18050640 | 4850 | 40 | 18441506 |
| final-100-layerfs-r2 | 4655 | 18050640 | 4850 | 40 | 18441506 |
| final-100-layerfs-r3 | 4655 | 18050640 | 4850 | 40 | 18441506 |
| final-500-layerfs-r1 | 10455 | 42189639 | 10871 | 200 | 43405159 |
| final-500-layerfs-r2 | 10455 | 42189639 | 10871 | 200 | 43405159 |
| final-500-layerfs-r3 | 10455 | 42189639 | 10871 | 200 | 43405159 |

## Resource scopes

Host CPU deltas bracket before → after-product and include in-loop observations. LayerFS container CPU is the runner command window. Native CPU brackets its workload; container lifetime peak memory also includes preparation/verification. Host CPU is not container-capped.

| Run | Host user CPU ns | Host system CPU ns | Host after-product RSS bytes | Container CPU ns | Container lifetime peak bytes | Cleanup |
| --- | --- | --- | --- | --- | --- | --- |
| baseline-100 | 1020467792 | 820397583 | 50003968 | 2169003000 | 51171328 | {'status': 'PASS', 'wall_ns': 1039612417} |
| baseline-500 | 2468491000 | 1936639625 | 72974336 | 5183595000 | 101720064 | {'status': 'PASS', 'wall_ns': 1806727125} |
| baseline-native-100-ownership-fixed | None | None | None | 197219000 | 257748992 | {'status': 'PASS', 'container_removed': True} |
| baseline-native-500 | None | None | None | 537656000 | 426315776 | {'status': 'PASS', 'container_removed': True} |
| final-100-layerfs-r1 | 228620500 | 141109000 | 54132736 | 1151951000 | 73777152 | {'status': 'PASS', 'wall_ns': 1155532333} |
| final-100-layerfs-r2 | 218475417 | 134064083 | 53673984 | 1156633000 | 73904128 | {'status': 'PASS', 'wall_ns': 1068349500} |
| final-100-layerfs-r3 | 212607917 | 118821500 | 53903360 | 1170933000 | 73625600 | {'status': 'PASS', 'wall_ns': 1074512584} |
| final-100-native-r1 | None | None | None | 187911000 | 257695744 | {'status': 'PASS', 'container_removed': True} |
| final-100-native-r2 | None | None | None | 194704000 | 255479808 | {'status': 'PASS', 'container_removed': True} |
| final-100-native-r3 | None | None | None | 194222000 | 255672320 | {'status': 'PASS', 'container_removed': True} |
| final-500-layerfs-r1 | 525416000 | 298699542 | 78970880 | 2975770000 | 142307328 | {'status': 'PASS', 'wall_ns': 1947294667} |
| final-500-layerfs-r2 | 508033667 | 284085958 | 79216640 | 2906353000 | 149643264 | {'status': 'PASS', 'wall_ns': 1818795584} |
| final-500-layerfs-r3 | 481768291 | 265566291 | 80658432 | 3030107000 | 150265856 | {'status': 'PASS', 'wall_ns': 1794916042} |
| final-500-native-r1 | None | None | None | 523831000 | 428122112 | {'status': 'PASS', 'container_removed': True} |
| final-500-native-r2 | None | None | None | 557611000 | 429772800 | {'status': 'PASS', 'container_removed': True} |
| final-500-native-r3 | None | None | None | 523838000 | 433860608 | {'status': 'PASS', 'container_removed': True} |

## Source and fixture identities

| Run | Source | Product | Image | Input | Raw SHA-256 |
| --- | --- | --- | --- | --- | --- |
| baseline-100 | 8e9f68ea6b6397a1c2ff61dc85ad7aa8960a9a796f5ebd2090b635234ac1eb42 | 0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695 | sha256:3b517047d43e17dd46aa8bb753a24fd693b82ee71bed5c8c764088e5124c68c8 | 45387468bd668465fd8a70a977d66ba4a282c482695d55eb36d50708f1ef7067 | 74f08695b96b2d8bb0a8917c3777f6b9d80561902f11a040eef3c0ece2ede622 |
| baseline-500 | 8e9f68ea6b6397a1c2ff61dc85ad7aa8960a9a796f5ebd2090b635234ac1eb42 | 0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695 | sha256:3b517047d43e17dd46aa8bb753a24fd693b82ee71bed5c8c764088e5124c68c8 | 151db6973f2703895b2c3c018cce5a88436e11d9acefd17f4586aee2b5a6853c | 60ca4ebf5229a8995ff79aa94aadfc4b59c3fd5cc15459a4b1c7fbfe3f3c02cb |
| baseline-native-100-ownership-fixed | 8e9f68ea6b6397a1c2ff61dc85ad7aa8960a9a796f5ebd2090b635234ac1eb42 | 0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695 | sha256:3b517047d43e17dd46aa8bb753a24fd693b82ee71bed5c8c764088e5124c68c8 | 45387468bd668465fd8a70a977d66ba4a282c482695d55eb36d50708f1ef7067 | b064dd25e2d52f782571c70bae7cb58cb02dfa1f9759f4595b2d45a5454cb0a6 |
| baseline-native-500 | 8e9f68ea6b6397a1c2ff61dc85ad7aa8960a9a796f5ebd2090b635234ac1eb42 | 0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695 | sha256:3b517047d43e17dd46aa8bb753a24fd693b82ee71bed5c8c764088e5124c68c8 | 151db6973f2703895b2c3c018cce5a88436e11d9acefd17f4586aee2b5a6853c | f728bccda09e26d58c13fc6fa4ba0d2e80a06ee4227b0bb3cda5ff4d48d92f0e |
| final-100-layerfs-r1 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | f0041d1d8ae10828c3608adbcc09f947d18f32fc9068d3a8833dcbbc53ee5c2c | 92e36348b85eb4ef46271026e3bd94a9ee1c038f82725b2d4430ed1aeeae451d |
| final-100-layerfs-r2 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | f0041d1d8ae10828c3608adbcc09f947d18f32fc9068d3a8833dcbbc53ee5c2c | 03337ffdb1b6704b6ec0f4498e6d1c65c13b1f12fe2bcd6372077e0b3219e727 |
| final-100-layerfs-r3 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | f0041d1d8ae10828c3608adbcc09f947d18f32fc9068d3a8833dcbbc53ee5c2c | 4105bc6e8901e2ec3fc099aaa67162cf898a49d65f503811b4f2225787476f5b |
| final-100-native-r1 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | f0041d1d8ae10828c3608adbcc09f947d18f32fc9068d3a8833dcbbc53ee5c2c | d1b765112c27ece4cc3615c4c3409abdacfa3fba36a530557e87a45f550ad2a7 |
| final-100-native-r2 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | f0041d1d8ae10828c3608adbcc09f947d18f32fc9068d3a8833dcbbc53ee5c2c | 934247bb1722b96cd700018726f0b6e1a8d0458cacc43f69aac6fe6adadce36f |
| final-100-native-r3 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | f0041d1d8ae10828c3608adbcc09f947d18f32fc9068d3a8833dcbbc53ee5c2c | 36fe46be3e88f8252b78dfe5e45ea110707c4cd664859d7b68eb1a4d9a3ea7f1 |
| final-500-layerfs-r1 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | d7543b66ced83002785e4c1f5073de6e15b8b7a69746dd0ebbda74f79b24da27 | 3881b5741c221013a6d077ef7280fb4f274ea8eb08c7aa7894f695f4c2a60531 |
| final-500-layerfs-r2 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | d7543b66ced83002785e4c1f5073de6e15b8b7a69746dd0ebbda74f79b24da27 | 40ecea18a0477aad8f5988db3f21b57d95b04f663e5ad631c053af7dcda35f4e |
| final-500-layerfs-r3 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | d7543b66ced83002785e4c1f5073de6e15b8b7a69746dd0ebbda74f79b24da27 | 2adb0e463890a4c3a8756125497feffc14bc40047300165beccc2aaf0e1d0c95 |
| final-500-native-r1 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | d7543b66ced83002785e4c1f5073de6e15b8b7a69746dd0ebbda74f79b24da27 | 640baba72a992ddeea4b826efded0ea1ffbd0868844b924a3b3ca4160a48e409 |
| final-500-native-r2 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | d7543b66ced83002785e4c1f5073de6e15b8b7a69746dd0ebbda74f79b24da27 | 35c7a42f77dacba02287953643e1c8273fbf23c6ee35101706dd17e3b820e6e3 |
| final-500-native-r3 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | 77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114 | sha256:7a3f781be3a078448ff789e952d8bb7e95aa03a485e5755c7208002f5d046ad6 | d7543b66ced83002785e4c1f5073de6e15b8b7a69746dd0ebbda74f79b24da27 | 04bdfe063299b2f64937f036a75c757f54c0fd092aa92fd734619f6f929514fa |

## Evidence limits and separate correctness receipts

Performance collection PASS and the historical 15-second classifier are not acceptance against 500,000,000 ns / 1,000,000,000 ns. Performance does not establish Git correctness, issue closure or publication. Separate proof receipts below are retained without treating their durations as performance observations.

| Case | Status | Wall seconds | Source identity | Raw receipt |
| --- | --- | --- | --- | --- |
| git-tool-100-mixed-v4 | PASS | 17.493885 | 8e9f68ea6b6397a1c2ff61dc85ad7aa8960a9a796f5ebd2090b635234ac1eb42 | [baseline-proof-100/verification.json](raw/baseline-proof-100/verification.json) |
| git-tool-500-mixed-v4 | PASS | 35.827208 | 8e9f68ea6b6397a1c2ff61dc85ad7aa8960a9a796f5ebd2090b635234ac1eb42 | [baseline-proof-500/verification.json](raw/baseline-proof-500/verification.json) |
| git-tool-1-compact-v2 | PASS | 5.328414 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | [final-proof-1/verification.json](raw/final-proof-1/verification.json) |
| git-tool-10-compact-v2 | PASS | 7.136484 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | [final-proof-10/verification.json](raw/final-proof-10/verification.json) |
| git-tool-100-mixed-v4 | PASS | 11.110569 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | [final-proof-100/verification.json](raw/final-proof-100/verification.json) |
| git-tool-500-mixed-v4 | PASS | 19.781520 | 7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074 | [final-proof-500/verification.json](raw/final-proof-500/verification.json) |

Full proof receipts remain in report.json; their wall times are separate from performance. If the table is empty, no separate proof receipt was supplied to this derivation.

```json
{
  "declaration_status": "EXACT_RUN_SET",
  "declaration_raw": "final-declaration.json",
  "declaration_error": null,
  "missing_runs": [],
  "parse_errors": []
}
```

Legacy zero counters known to be unwired are null/unavailable in derived metric groups; raw receipts retain their original zeros. All metric groups, lifecycle phases, workload counters, resource observations, cache policies and identities are retained in [report.json](report.json).

## Raw evidence hashes

| Relative path | Bytes | SHA-256 |
| --- | --- | --- |
| baseline-100/perf.jsonl | 30066 | 74f08695b96b2d8bb0a8917c3777f6b9d80561902f11a040eef3c0ece2ede622 |
| baseline-500/perf.jsonl | 30200 | 60ca4ebf5229a8995ff79aa94aadfc4b59c3fd5cc15459a4b1c7fbfe3f3c02cb |
| baseline-native-100-ownership-fixed/cleanup.json | 43 | b894a30706ab5cbdc8c5c30f9d258e18647f80ad4dc189e0580e5707b783f88f |
| baseline-native-100-ownership-fixed/container.json | 6738 | 4db917a2e0e6df36d68373ef388aa48fc6cd9226a24ec9405bf04d45d78a9d1f |
| baseline-native-100-ownership-fixed/manifest.json | 5243 | f31f3cb8470740181e16ffaedb1fd04f76460fa1515885a8bf002a4a898fc493 |
| baseline-native-100-ownership-fixed/matched-identities.json | 2383 | 6012b49f59d09cef509dccee542c7cd597f49e8b836e115582b350855c426f15 |
| baseline-native-100-ownership-fixed/native-result.json | 3254 | b064dd25e2d52f782571c70bae7cb58cb02dfa1f9759f4595b2d45a5454cb0a6 |
| baseline-native-500/cleanup.json | 43 | b894a30706ab5cbdc8c5c30f9d258e18647f80ad4dc189e0580e5707b783f88f |
| baseline-native-500/container.json | 6738 | 3a09ef0fd4ef0670ce2096cd8ca208ef349d98186cb54fa342f278adb7957eba |
| baseline-native-500/manifest.json | 5243 | 41659fd97fd79376082d3f242bee7aeafeb3d8b102305401208744e82fc4e870 |
| baseline-native-500/matched-identities.json | 2383 | 12b3a9e8bf25dbc1517eec8b763fdb1bc43072c47e732a64707719560d9a7374 |
| baseline-native-500/native-result.json | 3278 | f728bccda09e26d58c13fc6fa4ba0d2e80a06ee4227b0bb3cda5ff4d48d92f0e |
| baseline-proof-100/verification.json | 35583 | 069a7280fe8df568afb82f922ee95125c9dbd63cc8e3920e021f71ba9f928725 |
| baseline-proof-500/verification.json | 35743 | 9ee37bcb569f23bafb0886ef032c0bc62f15cbf9c21ac1573c9208b7109273f2 |
| final-100-layerfs-r1/perf.jsonl | 30380 | 92e36348b85eb4ef46271026e3bd94a9ee1c038f82725b2d4430ed1aeeae451d |
| final-100-layerfs-r2/perf.jsonl | 30208 | 03337ffdb1b6704b6ec0f4498e6d1c65c13b1f12fe2bcd6372077e0b3219e727 |
| final-100-layerfs-r3/perf.jsonl | 30208 | 4105bc6e8901e2ec3fc099aaa67162cf898a49d65f503811b4f2225787476f5b |
| final-100-native-r1/cleanup.json | 43 | b894a30706ab5cbdc8c5c30f9d258e18647f80ad4dc189e0580e5707b783f88f |
| final-100-native-r1/container.json | 6738 | b662310b4ceb77d5d072083011e674156669c65f0edb210c338b69d38aa5b306 |
| final-100-native-r1/manifest.json | 5243 | a29b5ac01cb699e397983a5358a0ef69ac5bd22b68a97b956f1a67d05589195c |
| final-100-native-r1/matched-identities.json | 2383 | 842e1f554b6979dbba289c877a546b961c5a4578791543d2a72b2ac254841ea2 |
| final-100-native-r1/native-result.json | 3253 | d1b765112c27ece4cc3615c4c3409abdacfa3fba36a530557e87a45f550ad2a7 |
| final-100-native-r2/cleanup.json | 43 | b894a30706ab5cbdc8c5c30f9d258e18647f80ad4dc189e0580e5707b783f88f |
| final-100-native-r2/container.json | 6738 | 27145c0d9f67aaf5c263e7697b4041e0db56b6c8e1ab4e266a01edcc60768520 |
| final-100-native-r2/manifest.json | 5243 | 20f800e6cd35758eca817da25fdcf60b536b0ffd12e3915347c33dfdc9505564 |
| final-100-native-r2/matched-identities.json | 2383 | 842e1f554b6979dbba289c877a546b961c5a4578791543d2a72b2ac254841ea2 |
| final-100-native-r2/native-result.json | 3253 | 934247bb1722b96cd700018726f0b6e1a8d0458cacc43f69aac6fe6adadce36f |
| final-100-native-r3/cleanup.json | 43 | b894a30706ab5cbdc8c5c30f9d258e18647f80ad4dc189e0580e5707b783f88f |
| final-100-native-r3/container.json | 6738 | 2136a18c38be003fe117206a941331667a358b28b73d6c7ecff960454a7d117c |
| final-100-native-r3/manifest.json | 5243 | 9f00fed5eb36f624e38c1a062b222bb8e55f52f10690851f5d03f4732219cd7b |
| final-100-native-r3/matched-identities.json | 2383 | 842e1f554b6979dbba289c877a546b961c5a4578791543d2a72b2ac254841ea2 |
| final-100-native-r3/native-result.json | 3252 | 36fe46be3e88f8252b78dfe5e45ea110707c4cd664859d7b68eb1a4d9a3ea7f1 |
| final-500-layerfs-r1/perf.jsonl | 30538 | 3881b5741c221013a6d077ef7280fb4f274ea8eb08c7aa7894f695f4c2a60531 |
| final-500-layerfs-r2/perf.jsonl | 30355 | 40ecea18a0477aad8f5988db3f21b57d95b04f663e5ad631c053af7dcda35f4e |
| final-500-layerfs-r3/perf.jsonl | 30361 | 2adb0e463890a4c3a8756125497feffc14bc40047300165beccc2aaf0e1d0c95 |
| final-500-native-r1/cleanup.json | 43 | b894a30706ab5cbdc8c5c30f9d258e18647f80ad4dc189e0580e5707b783f88f |
| final-500-native-r1/container.json | 6738 | 0bcfbdba83008a39fb1bfbb6ea865daa5f80e3cc1fa35ddd5898dda1f5759232 |
| final-500-native-r1/manifest.json | 5243 | c15451cd08bcf9fb1f9ee455c6ef259e2462bdfced50ea7cc54375b620b91df5 |
| final-500-native-r1/matched-identities.json | 2383 | 156aa16c14f1b79aa3b96aa443d20e1bcbcfe954459d58a07521320931722cdd |
| final-500-native-r1/native-result.json | 3278 | 640baba72a992ddeea4b826efded0ea1ffbd0868844b924a3b3ca4160a48e409 |
| final-500-native-r2/cleanup.json | 43 | b894a30706ab5cbdc8c5c30f9d258e18647f80ad4dc189e0580e5707b783f88f |
| final-500-native-r2/container.json | 6738 | dfc44779fac4b52a460d509f812998e0c426bf03cff65850a32b3bbe528c9fcc |
| final-500-native-r2/manifest.json | 5243 | 12ecfc28fc3217c63e98ef4c88c9e6f2bcf4d35828989c5aa7c3db57777f50e0 |
| final-500-native-r2/matched-identities.json | 2383 | 156aa16c14f1b79aa3b96aa443d20e1bcbcfe954459d58a07521320931722cdd |
| final-500-native-r2/native-result.json | 3278 | 35c7a42f77dacba02287953643e1c8273fbf23c6ee35101706dd17e3b820e6e3 |
| final-500-native-r3/cleanup.json | 43 | b894a30706ab5cbdc8c5c30f9d258e18647f80ad4dc189e0580e5707b783f88f |
| final-500-native-r3/container.json | 6738 | 1db8d3a0b1266d496511d8372db84f7cfdc96854dd9695230215eca16c3afece |
| final-500-native-r3/manifest.json | 5243 | d8c7e61f139596aa1c377b1c9fed7bd7e1eab3ca0742115a0185bc55eb6cc1c9 |
| final-500-native-r3/matched-identities.json | 2383 | 156aa16c14f1b79aa3b96aa443d20e1bcbcfe954459d58a07521320931722cdd |
| final-500-native-r3/native-result.json | 3278 | 04bdfe063299b2f64937f036a75c757f54c0fd092aa92fd734619f6f929514fa |
| final-declaration.json | 2597 | f0f7eae1535538b573245b7e9bc86c02b7c68f383d28038d74d2283ddd69c9c0 |
| final-proof-1/verification.json | 35527 | cd613e73b7e8b3fc7fd9748c0118cf1fc5a6cf2915c8082357b451d5524a4ccc |
| final-proof-10/verification.json | 35648 | 4c9c6c3bd609326f8614835ab3e7a66ec93a4f260758118eb5dd4985b4109da8 |
| final-proof-100/verification.json | 35695 | a4c7c000fc14b6f29d1a66c3ba6cb103da96c68dc8d13de4614d15fcb35d88a7 |
| final-proof-500/verification.json | 35849 | 241ff23f9016d4a1e48d73420ed615e209a1ad6b28e1db411fddcac6e889b155 |
