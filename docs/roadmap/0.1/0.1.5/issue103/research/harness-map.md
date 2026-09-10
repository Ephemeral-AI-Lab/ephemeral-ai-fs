# Harness map: repository_history (stride3 / full157) + historical_access

Scope: an evidence-only map of the existing benchmark/qualification harness, written for
issue #103 integration work. **No source file was modified.** All paths are relative to
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb` unless absolute.

Working-tree state at audit time (relevant to "is this fresh?"): `crates/layerfs-content/src/object/references.rs`,
`crates/layerfs-content/src/tree/mod.rs` modified, `crates/layerfs-content/src/tree/compact.rs`
new, so any new build now has a different source seal than every recorded number below.

---

## 1. repository_history: workload definitions, profile ids, end-to-end commands

### 1.1 What the family is

`repository_history` is an **optional, explicitly selected** family that replays the frozen
DeepSeek repository history (157 original checkpoints) through the *real* public save/Commit
surface, then verifies every retained state against the original content/metadata oracles.
It reuses the pre-existing `storage_smoke` host-session machinery; it is not a separate
compiled workload.

- Family entry (registration + dispatch): `benchmark/fs-bench-pro/shared/repository_history.py`
  - `PROFILES` map, line 4: `{'stride-1': ('deepseek-full', 1), 'stride-3': ('deepseek-stride3', 3), 'stride-10': ('deepseek-stride10', 10)}`
  - `registry()`, lines 7–13: rows are `{'family':'repository_history','profile':<name>,'scenario':<scenario>,'optional':True,'default_status':'NOT_RUN_OPTIONAL','full157_indices':<tuple>,'state_count':<n>}`; the self-check asserts state counts `[157, 53, 17]`.
  - Selection formula: `sorted(set(range(1,158,stride)) | {157})` — so **stride-1 = indices 1..157 (157 states)**, **stride-3 = 1,4,7,…,157 (53 states)**, stride-10 = 1,11,…,151,157 (17 states).
  - Execution requires `--profile`; `--list`/`--self-check` construct nothing (lines 24–33, 37–40). `repository_history.py:41-44` forwards to `storage_smoke.main(['--storage-smoke', scenario, *remaining])`.
- Selected-profile fixture/oracle generator: `benchmark/fs-bench-pro/shared/deepseek_ten.py`
  - `PROFILES`, lines 10–18: `'deepseek-stride3': (tuple(range(1,158,3)), 'deepseek-stride3-v1', 'docs/roadmap/0.1/0.1.5/issue100/stride3-snapshot-contract.md')`; `'deepseek-stride10'` carries scenario `deepseek-stride10-v1` and contract `docs/roadmap/0.1/0.1.5/issue102/campaign-v1.md`.
  - `inputs(data, cache, deadline, profile)`, line 20+: validates the frozen manifest/tip, extracts each selected tree from `source.git` with `git cat-file --batch`, authenticates blobs, compares against the original oracle JSON, and caches an immutable 0555/0444 fixture dir (line ~64–70).
- **full157 does not use `deepseek_ten.py`.** It is the existing `deepseek-full` scenario inside
  `storage_smoke.py`: `deepseek_inputs(data, deadline, count=157)` at
  `benchmark/fs-bench-pro/shared/storage_smoke.py:75-119`, selected at
  `storage_smoke.py:477`.
- Rust-side session: `benchmark/fs-bench-pro/src/storage_smoke.rs`. The Python layer maps
  *all* selected DeepSeek scenarios onto the compiled `deepseek-full` session
  (`storage_smoke.py:281`: `session_case = "deepseek-full" if case in SELECTED_DEEPSEEK else case`),
  because the compiled session accepts explicit `step <index>` commands and the *selection
  and cardinality belong to the runner* (comment at `storage_smoke.py:279-280`). Case
  allow-list and step range: `src/storage_smoke.rs:824-836` (`deepseek-full` accepted) and
  `src/storage_smoke.rs:911-935` (index range 1..157 for `deepseek-full`, 1..5 otherwise).
  There is no `deepseek-stride3` case in the compiled workload.

### 1.2 Exact end-to-end commands

Build once per relevant change (macOS host, from the repo root, Docker Desktop running):

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
export LAYERFS_BENCH_IMAGE="$(python3 benchmark/fs-bench-pro/shared/runner.py --build-image)"
```

stride3 (53 retained states, original indices 1,4,…,157):

```bash
# performance (creates a fresh host Store, one Commit per selected state)
python3 benchmark/fs-bench-pro/shared/runner.py --family repository_history \
  --profile stride-3 --image "$LAYERFS_BENCH_IMAGE" \
  --output /absolute/new/stride3-run

# verification of THAT SAME measured Store (original content/metadata oracles)
python3 benchmark/fs-bench-pro/shared/runner.py --family repository_history \
  --profile stride-3 --image "$LAYERFS_BENCH_IMAGE" \
  --storage-verify-run /absolute/new/stride3-run
```

full157 (157 retained states, original indices 1..157):

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --family repository_history \
  --profile stride-1 --image "$LAYERFS_BENCH_IMAGE" \
  --output /absolute/new/full157-run

python3 benchmark/fs-bench-pro/shared/runner.py --family repository_history \
  --profile stride-1 --image "$LAYERFS_BENCH_IMAGE" \
  --storage-verify-run /absolute/new/full157-run
```

Equivalent lower-level route (what the stride3 campaign actually ran, see
`docs/roadmap/0.1/0.1.5/issue100/experiments40/stride3/commands.json`):

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke deepseek-stride3 \
  --image layerfs-bench-infra:<source-seal-16> --source-arm candidate \
  --output /absolute/new/stride3-run
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke deepseek-stride3 \
  --image layerfs-bench-infra:<source-seal-16> \
  --storage-verify-run /absolute/new/stride3-run
```

Legacy full157-only alias: `runner.py --deepseek-full ...` (runner.py:635-636).

### 1.3 macOS vs Linux Docker split, and required setup

macOS host owns (see `benchmark/AGENTS.md`, `docs/general/benchmark_rules.md`,
`docs/roadmap/0.1/0.1.5/full157-execution-contract.md` "Topology, budgets and measurement"):

- the SQLite Store: `<output>/<case>/host-runtime/store.sqlite` (`storage_smoke.py:259`);
- the SDK/coordinator: host binary `target/release/fs-benchmark-pro storage-smoke-session
  <host-root> <container-id> performance|verification <case> <input>`
  (`storage_smoke.py:282`);
- canonical construction/Commit publication and the physical spool
  (`TMPDIR=<host>/tmp`, `storage_smoke.py:277`);
- fixture/oracle preparation from `source.git`;
- all docker control (`docker cp`, `docker exec`, cgroup reads).

Linux Docker owns only the daemon + real FUSE + workload helper: `runtime.start_sample`
(`benchmark/fs-bench-pro/shared/runtime.py:345-384`) creates the container with
`--cpus 2 --memory 2GiB --memory-swap 2GiB --pids-limit 256 --device /dev/fuse
--cap-add SYS_ADMIN --security-opt apparmor=unconfined`, network bridge, and runs
`/usr/local/bin/layerfs-daemon-entrypoint`. The workload helper is invoked through the FUSE
mount `MOUNT=/workspace/storage-smoke` (`src/storage_smoke.rs:6-7`); host input trees are
copied in with `docker cp` (never bind-mounted) via `runtime.install_tree`
(`runtime.py:508-518`).

Setup required before a run:

1. `--build-host` → qualified host binary + identity (see §2).
2. `--build-image` (or the storage-smoke variant) → sealed container image whose labels carry
   the same source/product seals (`Dockerfile.layerfs`, `LABEL dev.layerfs.source-seal` …).
   `--build-storage-smoke-image` additionally sets `LAYERFS_BUILD_SELF_CHECK=0`
   (`runner.py:673-675`).
3. Fixtures:
   - full157: `/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data` (default `--data`),
     manifest `checkpoint-manifest.json` SHA256 `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`
     (verified locally), tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`, per-checkpoint trees
     under `source.git`, cached per-state inputs under `inputs/<sha>/`, oracles under
     `oracles/<sha>.json` (157 files).
   - stride3/stride10: prepared on cache miss under `--fixtures` (default
     `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-data`), dir name
     `<profile>-<sha256[:20]>`. **stride3 is already prepared**:
     `layerfs-storage-v3-data/deepseek-stride3-218c1b81ed39c763ca22/fixture.json`,
     SHA256 `3b2c12b682e4892837fc810969e4e90eaa8d756c0d6f738df50746ad478fb73e`
     (matches `docs/roadmap/0.1/0.1.5/issue100/stride3-fixture-results.md`). Its
     `full157_indices` are exactly `[1,4,7,…,157]`, 53 states, each row carrying
     `index` (1..53), `full157_index`, `sha`, `tree`, `input`, `oracle`, `oracle_sha256`.
     No `deepseek-stride10-*` / `deepseek-ten-*` cache exists yet.
4. Seals/timeouts: the runner/`storage_smoke` refuse a stale build or image
   (`storage_smoke.py:466`), refuse a mismatched verification custody chain
   (`storage_smoke.py:483-491`), and use `LIMITS = {..., 'deepseek-full': (14400, 300, 14400),
   'deepseek-stride3': (14400, 300, 14400), ...}`
   (`storage_smoke.py:27`) = (performance phase, per-command, verification phase) seconds.
   Free-disk preconditions: ≥50 GiB free, spool+staging ≤16 GiB, owned dir ≤32 GiB
   (`storage_smoke.py:344, 468`).

---

## 2. "Qualified source-isolated build" and the runner-owned measurement lock

### 2.1 Qualified source-isolated build

`runner.py --build-host` (`benchmark/fs-bench-pro/shared/runner.py:646-668`) does, inside the
lock:

1. `values = source_build_args()` (`runner.py:131-152`) — seals the whole harness and product:
   sha256 over `crates/`, `tools/`, `benchmark/fs-bench-pro/` (`.rs/.toml/.sh/.py/.sql`, minus
   `target`/`__pycache__`) plus root `Cargo.toml`/`Cargo.lock`/`Dockerfile.layerfs`, split into
   `LAYERFS_SOURCE_SEAL` (everything) and `LAYERFS_PRODUCT_SEAL` (only `crates/**`), plus
   `LAYERFS_SOURCE_COMMIT`, `LAYERFS_SOURCE_TREE`, `LAYERFS_SOURCE_DIRTY`,
   `WORKLOAD_SOURCE_SHA256` (`workload/main.rs`).
2. Builds with **an isolated target dir keyed by the source seal**:
   `build_target = benchmark-results/host-store/builds/<LAYERFS_SOURCE_SEAL>`
   (`runner.py:293` defines `HOST_ROOT`; `runner.py:648`) and
   `cargo +1.85.1 build --locked --release -j2 -p fs-benchmark-pro --target-dir <build_target>`
   (`runner.py:650`).
3. **Proves the binary is from the current schema source**: reads
   `crates/layerfs-layerstack-store/src/schema.rs` for `SCHEMA_VERSION`
   (`runner.py:655-658`), then runs the freshly built binary's `infra-schema-probe` in a temp
   dir and requires the observed linked schema to equal the source schema
   (`verify_linked_schema`, `runner.py:610-617`). A stale/mismatched build is refused.
4. Copies to `target/release/fs-benchmark-pro` and writes
   `target/release/fs-benchmark-pro.identity.json`
   (`runner.py:664-666`) containing all seals, `build_target`,
   `observed_schema_version`, `binary_sha256`, `platform`, `rust_toolchain`, `schema_sha256`.

Consumers re-verify the identity before doing work: `storage_smoke.py:463-467`,
`historical_access.py:70-77`, `runner.py:609-617`.

### 2.2 The runner-owned measurement lock

- Path: `$TMPDIR/layerfs-infra-measurement.lock` (fallback `/tmp`).
- Mechanism: `open(path, "a")` + `fcntl.flock(fd, LOCK_EX | LOCK_NB)` — an advisory exclusive
  OS file lock, non-blocking. **Acquire = open+flock; release = closing the fd** (context-manager
  exit, `lock.close()`, or process exit). It is not a queue; a contender fails immediately.
- Owners in this repo:
  - `runner.py:639-643` — `--build-image` / `--build-host` / `--build-storage-smoke-image`
    (raises `RuntimeError("another benchmark owns the measurement lock")`).
  - `runner.py:691-696` — every selected family run (`parser.error(...)` on contention).
  - `storage_smoke.py:459-460` — the whole storage-smoke/repository_history run (note: the
    `BlockingIOError` is **not** caught here, so contention surfaces as an exception/traceback).
  - `historical_access.py:158-159` — acquired, `BlockingIOError` caught by the outer `except`
    and recorded as FAIL; **it never waits** (`docs/roadmap/0.1/0.1.5/issue102/campaign-v1.md`).
  - `benchmark/fs-bench-pro/issue54_collect.py:72-75` — the collector takes `LOCK_EX`
    (blocking) around each child run and retains refusals in `lock-refusals.jsonl`.
  - `benchmark/fs-bench-pro/verify-selected.py:265-270`.
- Practical rule: run exactly one resource-sensitive owner at a time; a caller must not
  double-acquire (already inside `storage_smoke`), and `$TMPDIR` must be the same for queuing
  to work. Historical lock file observed on this host:
  `/var/folders/s4/xpkmz7wn6yq97w1ls_4f_dfc0000gn/T/layerfs-infra-measurement.lock`.

---

## 3. How the complete allocated Store size is measured

**Definition:** filesystem-allocated bytes = Σ over `store.sqlite`, `store.sqlite-wal`,
`-shm`, `-journal` of `st_blocks * 512`. **Not** `du` at the harness level, **not**
`page_count*page_size`, and **not** a logical length. `PRAGMA page_size/page_count/
freelist_count` and canonical byte totals are recorded alongside as *separate* diagnostics.

Code path (Rust, product-side, emitted by the host session):

- `benchmark/fs-bench-pro/src/storage_smoke.rs:608-660` — `fn storage(store, label)`:
  - lines 614–622: loop over `["", "-wal", "-shm", "-journal"]`, `allocated += m.blocks() * 512`,
    `apparent += m.len()` (`std::fs::Metadata::blocks() == st_blocks`).
  - line 622: `store.canonical_storage()` (canonical objects/bytes).
  - lines 624–636: `PRAGMA page_size`, `page_count`, `freelist_count`.
  - lines 637–658: emits one `storage-smoke-allocation` record with fields
    `store_apparent_bytes`, **`store_allocated_bytes`**, `database_logical_bytes`,
    `database_allocated_bytes`, `sidecar_logical_bytes`, `sidecar_allocated_bytes`,
    `page_size_bytes`, `page_count`, `freelist_page_count`, `canonical_bytes`, `canonical_objects`.
  - lines 659–662: hard budget — apparent or allocated > 16 GiB fails the run.
- Emission points: `after-init` (`src/storage_smoke.rs:865`), **`step-<index>` after every
  checkpoint** (`src/storage_smoke.rs:792`), and **`after-end`** (`src/storage_smoke.rs:1078`).
  The *retained* number is the `after-end` receipt, taken after clean coordinator closure and
  before any verifier-created records (`docs/roadmap/0.1/0.1.5/full157-execution-contract.md`).
- A second, independent Python-side accounting exists for directories:
  `storage_smoke.py:41-48` `disk(root)` → `{"apparent_bytes", "allocated_bytes"}` (also
  `st_blocks*512`), recorded as `record["host_runtime_disk"]` (`storage_smoke.py:340`),
  `record["spool_disk"]` (line 341), `result["retained_disk"]` (line 380), plus
  `record["container_staging_allocated_bytes"] = du -sk /input * 1024`
  (`storage_smoke.py:342-343`). These are budget/temp-storage observations, not the headline
  Store number.

**Units: decimal bytes.** Examples:

- In-repo recorded receipt (stride3, §6): `docs/roadmap/0.1/0.1.5/issue100/experiments40/stride3/result.json`
  → `"layerfs_allocation": {"kind":"storage-smoke-allocation","label":"after-end",
  "store_allocated_bytes":100700160,"database_allocated_bytes":100700160,
  "store_apparent_bytes":84844544,"page_size_bytes":4096,"page_count":20714,
  "freelist_page_count":0,"sidecar_allocated_bytes":0,"canonical_bytes":595887438}`.
  So 100,700,160 B allocated vs 84,844,544 B logical/apparent for the same file.
- `docs/roadmap/0.1/0.1.5/issue100/retained-full157-results.md:60` tabulates the field name
  `store_allocated_bytes | 134,246,400` for the full157 arm.
- **`benchmark-results/` contains no recorded measurement receipts.** It holds only build
  caches (`benchmark-results/host-store/builds/<seal>/`), prepared-input caches
  (`prepared/`, `fixture-identities/`), and `quarantined-issue102/`. `host-store/samples/` is
  empty and `git ls-files benchmark-results` returns nothing (all untracked). Any "recorded
  measurement" must be read from the docs or the external evidence dirs listed in §6.

---

## 4. Original content/metadata oracles and retained-state verification

### 4.1 Where the oracles come from

Frozen source of truth: `/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data`.

- `checkpoint-manifest.json` — SHA256 `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`,
  tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`, 157 checkpoints (pinned in
  `storage_smoke.py:22-23` as `MANIFEST_SHA` / `SOURCE_TIP`).
- `source.git` — the real Git objects used to re-authenticate every tree (`git ls-tree -rlz
  --full-tree <sha>` must hash to `manifest_sha256`) and every changed blob
  (`sha1("blob <len>\0"+body) == oid`): `storage_smoke.py:88-108`, `deepseek_ten.py:33-56`.
- `oracles/<sha>.json` — the **original content + metadata oracle** for each checkpoint: a map
  `hex(path) -> [mode, size, sha256(body)]` for files plus `["40755",0,"-"]` for every
  implied directory. Built/validated in `storage_smoke.py:101-115` (full157) and
  `deepseek_ten.py:57-63` (selected profiles), and re-derived independently from Git bytes at
  verification time (a mismatch raises `cached oracle differs from independently authenticated
  Git blobs` / `selected original full-byte oracle mismatch`). 157 files exist in
  `deepseek-history-data/oracles/`.
- Cached per-state inputs `inputs/<sha>/` carry `manifest.tsv`, `previous.tsv`, `blobs/` and a
  receipt; seals are re-checked on every run (`storage_smoke.py:95-100`).

### 4.2 What verifies the retained states

`storage_smoke.py:321-335` (verification mode of the same `run_case`):

1. Send `verify <identity>` to the same host session (`storage_smoke.py:323`; Rust handler
   `src/storage_smoke.rs:1019-1052`): forks a `verify-*` branch from that CommitId (or the
   genesis layer for `initial`), mounts a real FUSE workspace, runs
   `fs-benchmark-workload storage-smoke-observe MOUNT /input/observed.tsv`.
2. `docker cp <container>:/input/observed.tsv <output>/observed-<index>.tsv`
   (`storage_smoke.py:325`) and parses `kind,size,digest,path` lines.
3. Loads the oracle `Path(row["oracle"])` and requires **exact equality** of the whole
   map; on mismatch writes `mismatch-<index>.json` and raises
   `RuntimeError("historical oracle mismatch")` (`storage_smoke.py:331-334`).
4. Records `{"index", "identity", "status":"PASS", "verified_entries", "verified_bytes"}`
   (`storage_smoke.py:335`).

Index mapping: the row list for performance is `fixture["states"]`
(`storage_smoke.py:299`) — for stride3 each row has `index` (1..53 campaign ordinal, used in
the `step <index>` protocol and the `step-<index>` allocation label) and `full157_index`
(1,4,…,157, the original manifest index that selects the input dir and oracle). For full157
`index == full157_index == 1..157` (`storage_smoke.py:116`). Verification iterates the
**performance records**, i.e. exactly the states that were actually created
(`storage_smoke.py:299, 322`), and `storage_smoke.py:318-319` already requires
`created == true` for every stride3/stride10 state.

Custody before reopen: `storage_smoke.py:486-491` re-hashes each `case/host-runtime/store.sqlite`
against `performance-manifest.json`; a changed measured Store is refused
("measured Store changed before historical reopen").

Sealed fixture paths + hashes:

| Fixture | Path | SHA256 |
| --- | --- | --- |
| Frozen Git/manifest source (full157) | `/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data/checkpoint-manifest.json` | `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271` |
| stride3 prepared fixture | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-data/deepseek-stride3-218c1b81ed39c763ca22/fixture.json` | `3b2c12b682e4892837fc810969e4e90eaa8d756c0d6f738df50746ad478fb73e` |
| historical_access sealed Store (schema9, 157 states) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full/host-runtime/store.sqlite` | `f323de0e0f9ae1030efc142402bc033ad427134dd8c69f21eb5b5d6ef7426eb7` (re-verified locally during this audit) |

(The stride10/ten caches do not exist yet; they are built on demand.)

---

## 5. historical_access: cases, 15-second contract, invocation

- Fixture/manifest: `benchmark/fs-bench-pro/families/historical_access/fixture.json`
  (schema `historical-access-v2`, `contract_commit 2192797ec`).
- Driver: `benchmark/fs-bench-pro/shared/historical_access.py` (+ shims
  `families/historical_access/perf.sh` → `--mode performance`,
  `verify.sh` → `--mode verification`).
- Contract doc: `docs/roadmap/0.1/0.1.5/issue101/historical-access-v2.md`;
  results: `.../issue101/results.md`.

**The 15-second contract** (`historical_access.py:18, 148-153, 184-192`):

- `LIMIT_NS = 15_000_000_000` is a **single hard end-to-end envelope per case, including
  preparation and teardown**. `main` receives the process entry timestamp
  (`runner.py:621,627-628` passes `ENTRY_STARTED_NS`), and `selected()` computes
  `end = now + (LIMIT_NS - elapsed_since_entry)`; the supervised worker gets
  `work_end = end - 3` (⇒ ~12 s working deadline), and `deadline_status` is FAIL unless
  `outer_wall_ns < LIMIT_NS`. `completion.json.total_including_receipts_ns` must also be
  `< LIMIT_NS`; the verifier re-checks that (`historical_access.py:92`).
- The measured phase is one public operation, timed by the workload:
  `access_operation_ns` (`benchmark/fs-bench-pro/workload/storage_smoke.rs:388`, timer started
  at line 374 and stopped at 387, around the actual `stat`/`read_dir`/`read_exact_at`; the
  oracle fields `access_mode/size/mtime/mtime_nsec[/sha256/names]` are printed at 390–395).
- One measurement lock acquisition, `LOCK_NB`: contention ⇒ FAIL, it does not queue
  (`historical_access.py:158-159`).

**Eleven performance cases** (`fixture.json`; asserted at `historical_access.py:31`,
`test_historical_access.py:7`), covering original indices **1, 157, 65, 57**:

| case id | full157 idx | operation | path | offset/length |
| --- | --- | --- | --- | --- |
| ha-stat-old-v2 | 1 | stat | README.md | 0/0 |
| ha-stat-head-v2 | 157 | stat | README.md | 0/0 |
| ha-directory-old-v2 | 1 | directory | . | – |
| ha-directory-head-v2 | 157 | directory | . | – |
| ha-small-old-v2 | 1 | read | README.md | 0/843 |
| ha-small-head-v2 | 157 | read | README.md | 0/2201 |
| ha-range-history-cold-v2 | 65 | read | pnpm-lock.yaml | 537371/6421 |
| ha-range-history-warm-v2 | 65 | read | pnpm-lock.yaml | 537371/6421 |
| ha-full-head-cold-v2 | 157 | read | pnpm-lock.yaml | 0/841964 |
| ha-full-head-warm-v2 | 157 | read | pnpm-lock.yaml | 0/841964 |
| ha-metadata-worst-v2 | 57 | stat | `.agents/notes/implemented/architecture/2026-07-19-gui-layering-and-rpc-protocol.zh.md` | 0/0 |

Every case pins a `commit_id` from the sealed Store plus `original_oracle_sha256`; warm cases
require exactly two workload outputs and `access_completed_count == 1`
(`historical_access.py:36-50`). Read cases additionally re-check full-file mode/size/mtime
after the pure read timer (`historical-access-v2.md` "Read cases additionally check…").

**Invocation:**

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --family historical_access --list

python3 benchmark/fs-bench-pro/shared/runner.py --family historical_access \
  --case ha-small-head-v2 \
  --store /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full/host-runtime/store.sqlite \
  --image "$LAYERFS_BENCH_IMAGE" \
  --output /absolute/new/ha-performance

python3 benchmark/fs-bench-pro/shared/runner.py --family historical_access \
  --case ha-small-head-v2 \
  --store /Users/…/retained-full157-1/deepseek-full/host-runtime/store.sqlite \
  --image "$LAYERFS_BENCH_IMAGE" --mode verification \
  --performance /absolute/new/ha-performance/result.json \
  --output /absolute/new/ha-verification

# all eleven, serially, each with its own 15 s envelope
python3 benchmark/fs-bench-pro/shared/runner.py --family historical_access \
  --all --store … --image "$LAYERFS_BENCH_IMAGE" --output /absolute/new/ha-all
```

The Store is **never built by this family**; it is copied byte-for-byte into the fresh output
(`runtime.closed_store_copy`, `runtime.py:526-549`, which refuses SQLite sidecars, re-hashes
master and copy, and runs `PRAGMA quick_check`) and must hash to
`fixture.store_sha256` or the run is `NOT_READY: incompatible history Store`
(`historical_access.py:99-101`). Verification binds one selected performance receipt via
`--performance` + its `manifest.json`/`completion.json` seals
(`historical_access.py:82-96`). Note: `historical_access` is **not** in the frozen issue102
registry (`docs/roadmap/0.1/0.1.5/issue102/mandatory-registry.jsonl` lists only the 17 host
families); campaign-v1.md declares adding it prospectively.

---

## 6. Recorded reference numbers (and whether they are historical or fresh)

| Quantity | Value | Evidence | Status |
| --- | --- | --- | --- |
| Product full157 allocated (retained candidate) | **134,246,400 B** (logical 130,990,080 B) | `docs/roadmap/0.1/0.1.5/issue100/retained-full157-results.md:5,19,60`; receipt `layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full/performance-result.json:270` (`store_allocated_bytes`) | Historical issue100 campaign (2026-09-10), reused under the applicability audit `issue100/baseline-applicability-45mb.md`; product seal differs from current tree |
| same campaign: released control | 184,582,144 B | `retained-full157-results.md:17`; `layerfs-issue100-evidence/comparison-control.json` | Historical |
| same campaign: initial v015 candidate | 201,371,648 B | `retained-full157-results.md:18`; `layerfs-issue100-evidence/initial/deepseek-full/performance-result.json` | Historical |
| Earlier "previously measured retained product" full157 | 134,246,400 B (and 129,937,408 B fresh VACUUMed offline control) | `issue100/40mb-offline-full157-results.md:13,14` | Historical, offline context |
| Product **stride3** allocated (53 states) | **100,700,160 B** (logical 84,844,544 B) | `issue100/stride3-comparison-results.md:3,20`; `issue100/experiments40/stride3/result.json` (`layerfs_allocation.store_allocated_bytes`); raw run `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-comparison/` | Fresh (2026-09-10), product seal `24cde1dc…`, source commit `72c2ed688` on `codex/issue100-40mb-experiments` |
| Matched Git53 packed | 49,332,224 B (apparent 48,951,284 B) | `stride3-comparison-results.md:20`; `experiments40/stride3/result.json` `git_packed` | Fresh, same campaign |
| Offline optimized **53 states** | **54,382,592 B** | `docs/roadmap/0.1/0.1.5/issue100/experiments40/ordered-optimization/content/result.json:13-14` (`candidate_disk.allocated_bytes` == `apparent_bytes`); summarised `ordered-optimization-results.md:5,16,18`, `optimization-checklist-and-experiment-ledger.md:7,22,33,163`, `0.1.5/README.md:37` | Offline diagnostic copy (single packed `candidate.sqlite`), **not** a product/Store measurement |
| Offline optimized **157 states** | **65,957,888 B** | `issue100/experiments40/ordered-optimization/content/full157/result.json:13-14`; `allocated_saved: 6012928` line 16; same doc refs as above | Offline diagnostic copy, archive/reference-only (severe small-read amplification); **this is the "65.96 MB" reference, not a promised online allocation** (`issue102/integrated-candidate-v2.md:27-30`) |
| Other offline prototypes (context) | 41,648,128 B (10-state combined), 107,958,272 B (full157 fixed D) | `issue100/40mb-experiment-results.md`, `issue100/40mb-offline-full157-results.md:13` | Offline only |

Caveats: (a) each of these was produced by a *different* source/product seal; none was
produced by the current working tree; (b) the 53-vs-157 comparison in
`stride3-comparison-results.md:33-44` is explicitly a different-cardinality iteration-cost
comparison, not a paired speedup; (c) there is **no 66,000,000-byte target recorded anywhere
in the repository** — the only ≈66 MB number in the repo is the ten-state line
`docs/roadmap/0.1/0.1.5/issue100/handoff-45mb.md:89` ("Existing v0.1.5 | 66,105,344 apparent |
66,035,712 allocated"), which is a different workload.

---

## 7. Existing harness features for compaction, timing, resources, temp storage

- **Optional compaction / VACUUM / repack: not present.** No `VACUUM`, repack, GC or
  "compact the Store" flag exists in `runner.py`, `storage_smoke.py`, `historical_access.py`,
  `src/storage_smoke.rs`, or the workload. The `compact(...)` helpers in
  `benchmark/fs-bench-pro/workload/ordinary_workloads.rs:14-30` and the
  `store-footprint-*-low-v1` controls (`families/store_footprint/mod.rs:44-77`) are *fixture
  size profiles*, not storage compaction. The only related mode is
  `--storage-compat-run` (`storage_smoke.py:394-441`): it copies a prior closed 64 KiB-layout
  Store with `runtime.closed_store_copy`, re-verifies it in place, and asserts the page size is
  still 65536 — a compatibility check, not a compaction, and it is explicitly
  `allocation_comparison_eligible: False`.
- **Save/Commit timing:** `fn timed()` (`src/storage_smoke.rs:552-600`) wraps each phase and
  emits `storage-smoke-phase` with `phase`, `elapsed_ns`, `success`, `physical_storage`,
  `host_cpu_ns`, `host_rss_bytes`, `host_lifetime_peak_rss_bytes`, `host_footprint_bytes`,
  `host_disk_read_bytes`, `host_disk_write_bytes`. Phases include `init`, `fork`, `mount`,
  `exec`, `sdk-edit`, `commit`, `verify-mount`, `verify-end`, `end`, and
  `access-measured` (historical_access). Per-state public `exec_ns`/`commit_ns`/`paired_ns`
  and `step_wall_ns` are assembled by Python (`storage_smoke.py:336`).
- **Per-Commit allocation:** `storage(store, "step-<index>")` after **every** checkpoint
  (`src/storage_smoke.rs:792`) — the per-state allocated growth series is already available.
- **Resource use:** cgroup before/after each step plus `memory.stat` categories
  (`storage_smoke.py:235-241, 295, 337-339`), OOM/swap gate (line 338), host RSS via the
  phase records, and `docs/…/full157-execution-contract.md` budgets (Store ≤16 GiB,
  staged/spool ≤16 GiB, owned dir ≤32 GiB, host free ≥50 GiB, RSS ≤8 GiB).
- **Temporary storage:** `record["spool_disk"] = disk(<host>/tmp)` (host physical spool),
  `record["container_staging_allocated_bytes"] = du -sk /input` (staged fixture),
  `record["host_runtime_disk"] = disk(<host>)`, `result["retained_disk"]`
  (`storage_smoke.py:340-345, 380`), plus the Commit diagnostics string carrying
  `edit_spool_allocated_bytes`, `physical_spool_allocated_bytes/peak` (e.g.
  `performance-result.json` `commit_diagnostics`).
- **Diagnostics/admission flags:** every selected storage-smoke run records
  `admission_eligible: False`, `cache_profile: fresh-store-existing-os-cache-uncontrolled`,
  `resource_profile: …` (`storage_smoke.py:495-501`) — these are exploratory qualifications.
- **Separate synthetic family** `store_footprint` (`families/store_footprint/`, runner family
  `store_footprint`, `TIMERS` entry `product_call_sum_ns` at `runner.py:56`) measures
  footprint for 100k-file / 500 MB synthetic controls, but it uses logical file length
  (`std::fs::metadata(store_path).len()`, `src/main.rs:1821`) and its own
  FIXTURE/PERFORMANCE/VERIFICATION schemas — it is **not** the repository_history allocation
  metric and cannot be used as evidence for full157/stride3.

---

## Not present in the repo (explicit)

- No `66,000,000`-byte target statement, and no "allocated Store ≤ 66 MB" gate.
- No cross-run persistent Store-size ledger inside `benchmark-results/`; that directory
  contains only build/prepared-input caches.
- No `deepseek-stride10-*` or `deepseek-ten-*` prepared fixture cache on this machine.
- No harness compaction/VACUUM/repack option, and no online (public Store) measurement of the
  54,382,592 / 65,957,888 byte offline layouts.
- `historical_access` is not registered in `docs/roadmap/0.1/0.1.5/issue102/mandatory-registry.jsonl`.
