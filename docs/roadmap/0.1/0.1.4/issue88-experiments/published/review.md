# Independent prospective experiment and custody review

Issue88 was read in full, together with the full157 M4.5 contract and applicable benchmark hosting instructions. The previous contract's no-new-encoding restriction describes the historical diagnostic; issue88's explicit new owner authority permits isolated retained-content encoding experiments. It does not authorize mutating original Stores, historical evidence, dependencies, production rollout or unreviewed simultaneous treatments.

Discovery checkout is `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue88-experiments`, branch `codex/issue88-encoding-experiments`, initial source `eb7050603c5ee97a02dfa0d5357619079d25ab51`. The root owner serializes all builds, encoders and product measurements with the existing measurement lock. This reviewer runs only product-free validator tests, not an encoder, Store scan or benchmark.

## Controls that identify an actual treatment

S1 must compare identical canonical records and original group membership/order with unchanged codec settings. Origin collection by executing the exact current tree editor over authenticated prior/new entries is **actual offline tree-engine reconstruction**, not historical producer-observed provenance. Every reconstructed root must equal its expected retained root; one mismatch invalidates the screen. Origins must be available strictly before the target checkpoint and resolve to FULL in the **candidate representation**, not merely FULL in the historical control index. No hypothetical origin may be chosen by final-tree similarity. Missing origin, splits/merges without valid origin, inadmissible bases and limits fall back FULL.

S2 needs identical current canonical identities in FULL and prior-prefix arms, explicit chronological candidate ownership and FULL fallback. S3 changes the canonical unit; it cannot inherit S2's canonical-identity-set equality. It instead needs exact preserved filesystem-state/oracle equality and a same-unit FULL control if attributing a result specifically to prefix compression. Compare complete images for granularity effect, including lost chunk reuse and wrappers. The S2/S3 codec, predecessor availability, depth and decoded-byte policies must remain equal where comparable or be explicitly declared treatment differences.

For each checkpoint, selected physical bases must already be admitted. Same-checkpoint bases require a specifically declared intra-checkpoint chronological provenance rule; final pack order or first-retained checkpoint alone cannot establish it. The simplest strict screen allows only earlier-checkpoint bases. Recurrent content should CAS-select its existing representation; it must not silently receive a better new representation unless replacement is a separately declared treatment.

## Allocation and encoded-potential boundaries

Every screen partitions selected record bodies, physical-base-only records, record/pack framing, dictionaries and index/metadata once. If only a content/structural subset is represented, its scope is explicit and omitted costs are unavailable, not a complete zero-overhead Store. Physical-base-only bodies cannot also be present in the selected logical-body total. Shared compressed groups remain indivisible measured units; no proportional per-record compressed savings.

Offline encoded bytes do not prove134221004 allocated Store bytes. The result envelope deliberately sets complete `allocated_store_bytes` to null with a reason and labels claims `offline_encoded_potential`. If a prospective fully materialized image is also created, its actual logical/allocation/stat measurements need a separately named artifact and cannot substitute for synchronous public acknowledgement allocation. Neither copied allocation nor final post-close allocation is the old acknowledgement-time control.

## Read/edit evidence and cheap rejection

A fresh application-level decoded-object cache is not a cold OS or cold disk. Label read conditions `fresh_application_decode_cache_os_cache_uncontrolled` or `warm_application_decode_cache_os_cache_uncontrolled`; do not purge or claim OS coldness without a separately declared method. Missing read measurements remain null with a reason, especially an S1 representation-only screen. A chain decoded-byte cap is a resource bound, not latency evidence.

Before a costly public-path candidate, the cheap offline correctness/read checks should cover exact prospective thresholds and actual edits: one-byte/localized replacement, repeated content recurrence selecting the existing identity, shared content across files, growth/shrink crossing262144 payload bytes in both directions, and small-range/full-file reconstruction through maximum permitted chains. Full canonical framing is charged separately when the codec uses raw payload. Original source-size/reference and synthetic boundary populations must remain separate in reporting. Synthetic favorable prefix sizes are capability checks only.

Promising screens proceed through the three approved existing smokes only after a narrowly scoped synchronous prototype has a frozen identity and an explicit concrete promotion rule. Those public-path smokes cannot be emulated by direct offline internal calls. Negative screens stop cheaply with their exact measurements; they need not force a rewrite merely to produce a product benchmark. No favorable seed selection or threshold change after observing output.

## Runnable result-envelope validator

`validate_results.py` is an adapter boundary around native engine outputs, not a second storage implementation. Root-owned orchestration can emit an `issue88-screen-envelope-v1` JSON with raw artifact hashes after a run. A separate expected JSON, frozen before execution, supplies exact stage, contract hash, identity, settings, population, state count, dependency caps, same-canonical-population flag, arm scope, lineage provenance and resource limits.

The validator hashes every referenced artifact and rejects path escape, size/hash changes, identity/settings drift, unmatched same-unit canonical populations, incomplete target/byte authentication, future/missing/cyclic bases, excessive dependency work, physical-byte conservation errors, fabricated offline allocation, unsupported cold-cache claims and resource/cleanup/custody failure. It does not reconstruct content itself: execution of real target/oracle/dependency validation and source review must precede the envelope. A self-declared successful boolean is not independent product correctness evidence.

Run:

```sh
python3 docs/roadmap/0.1/0.1.4/issue88-experiments/validate_results.py --self-test
python3 docs/roadmap/0.1/0.1.4/issue88-experiments/validate_results.py RESULT_ENVELOPE.json FROZEN_EXPECTED.json
```

The synthetic test accepts one complete envelope and rejects ten accounting, custody, chronology, scope/cache and resource errors. No real encoding run is implied by that test. Failed envelopes remain on disk and the CLI returns failure; do not rewrite them to PASS after a correction. Corrected implementations require a new source/binary/contract-compatible attempt identity and a new disposable output directory.

## Review status

The above conditions are the independent pre-execution review criteria. The exact root-owned prospective contract and real screen outputs must be reviewed before describing an empirical candidate as passed. At this writing no real experiment result has been reviewed or accepted by this reviewer; implementation/runs belong to the root and stage owners. Original artifacts remain untouched.

## Contract-v1 review at2b7eb7541

Read `contract-v1.md` in full before retained-content encoding. Its ordered S1→S2→S3 design, strict earlier-checkpoint bases, candidate-FULL S1 anchors, frozen S2/S3 codec3/window20/depth4/closure1MiB, cache labels, original-evidence protection and serialized execution are sound. The following concrete details must be frozen in committed code/settings or an additive contract supplement before the relevant run:

1. S3's exact experimental canonical tag/version/length encoding, so canonical identities and framing counts are reproducible. “Experimental version” alone is insufficient.
2. Exact FULL/PREFIX record, locator/index and container framing for S2/S3. State whether existing21-byte chunk canonical framing is implicit reconstruction or physically stored; never charge it twice or omit required type/length/base identifiers. The32-byte base-ID decision must compare complete otherwise-identical record forms.
3. Deterministic synthetic bytes/seed, localized-edit offset/length, cross-file and A→B→A recurrence order, and threshold growth/shrink bytes. A committed exact fixture function plus source hash can freeze these without verbose contract data dumps.
4. S1's planned16MiB matcher-work/512-trial limit resets per original pack, not necessarily the original admission-batch grouping. An additive supplement must identify this as an offline screen work policy rather than claim historical batch-policy equivalence.

These are implementation-freeze requirements, not requests for new owner permission. The root can resolve them autonomously before execution. Actual code/executable identities must be recorded after these details are fixed. No numerical promotion gate should be invented after inspecting results; a qualitative promising/reject disposition must give complete byte/cost evidence and freeze any next public prototype prospectively.

## Executable/source review and initial S1 observation

The committed `screen-details-v1.md` resolves the canonical layout, physical framing, deterministic fixture and S1 per-original-pack budget freeze questions above. The content encoder's raw-SHA256/exact-length round trip logically authenticates the canonical bytes when joined to the sealed extractor's exact canonical ID; reporting correctly avoids claiming an independently executed decoder BLAKE3 calculation.

A material extractor boundary issue was identified before S3 encoding: small→large transitions must not use prior-file chunks merely because the same chunk identity was selected globally through another file. The contract specifies a representation-type mismatch FULL fallback. Root was notified to constrain old chunks to an actually large prior file or authenticate that no existing manifest row is affected; any changed extraction must have new sealed output identity.

Content encoder review checked native-prefix lifetimes, context resets, frame/header bytes, chronological selected-index bases, iterative cycle/depth/whole-closure validation, output append restoration after reconstruction seeks, winner byte comparison and all-target SHA256 checks. Two format checks were requested before freezing: reject concatenated Zstd frames when one frame is specified, and validate the container magic/version/header size before trusting reads. The encoder owner reports both checks added, plus content-size validation and a same-kind prior-domain guard. No build or encoder was run by this reviewer.

S1 `issue88-s1-bb36062-1` completed its representation screen. Independent CSV arithmetic confirms9918 groups and279724 authenticated records; control78792537 B and selected56857102 B differ by21935435 B, with4585 DELTA records. All157 reconstruction rows report identical roots; candidate frame offsets are contiguous and exactly cover the56857102-byte output. Canonical94362335 B remains unchanged. The tool source verifies control encoding byte-for-byte against each original group, origins through actual offline engine execution, strictly prior bases that remain FULL in the candidate, and every selected record reconstruction. This is a27.84% structural encoded-group reduction, not measured allocated Store saving or public-path timing. Final artifact/resource custody and subsequent stages remain separate acceptance steps.

## Independent S2/S3 empirical acceptance

Reviewed completed `issue88-s2-b9abb13-1` and `issue88-s3-b9abb13-1`. Each result's four listed artifact hashes and logical lengths independently match. Source tool/library identities match the frozen `content-expected.json`; native library contents, contract hashes and input2 manifest hashes independently match. Input1 was preserved and superseded before encoding to resolve the identified threshold-predecessor bug.

A metadata-only CSV pass independently checked every target:86412 S2 and77487 S3 unique consecutive ordinals; every declared prior exists in the already seen earlier-checkpoint set and shares its unit kind; prefix winners strictly beat FULL after32-byte base framing; depths are≤4 and closures≤1048576 B. FULL and selected offsets are contiguous and exactly cover their complete containers. S2's maximum observed closure is163840 payload B; S3's is1027939 B. The encoder itself round-tripped all FULL/candidate frames and then reconstructed/hash-checked every selected object with its actual selected base closure. This review did not duplicate decoding or encoding.

| Screen | Complete FULL container bytes | Complete selected container bytes | Interpretation |
|---|---:|---:|---|
|S2 current chunk units|243582540|102306097|Canonical-preserving raw-content/prefix screen; metadata-value chunks excluded|
|S3 bounded whole-file/hybrid units|267114049|99824523|Changed canonical-unit payload screen; regenerated inode/metadata structures excluded|

The additional S3 selected-container gain is only2481574 B versus S2, while its unique raw input population grows by130552367 B. That does not make S3 incorrect, but it **reverses the earlier priority for a whole-file canonical rewrite**: the evidence favors retaining the existing content unit and pursuing the substantial actual-prior native-prefix gain first. S3's unknown wrapper/index savings cannot be assumed to compensate for its canonical migration and larger read granularity. Both payload containers remain above the hypothetical70MB payload budget, and neither is a complete134MB Store.

S1's21935435 B structural-group reduction and S2's content-container improvement are independent screens, not a measured combined Store saving. Different grouping, record framing, index/metadata layout, physical base policy and public writer scheduling prevent arithmetic addition from becoming a product claim. A combined representation requires its own complete image and synchronous allocation evidence.

The six read observations per content screen are fixed first/largest/deepest probes, not a latency distribution or equal-byte matched workload between S2/S3. “Fresh” resets decoder/application decoded cache while OS cache remains warmed/uncontrolled. The warm observation is a direct one-object cached Python slice/return, not SDK/filesystem/read-path latency. S3's largest-file small-range probe decodes333145 B to return4096 B; S2's largest-chunk probe decodes32768 B, but the different content and chain make a timing ratio misleading. Process wall about36.36s/35.08s and peak RSS53067776/55443456 B are offline process observations, not public Commit costs. Portable per-process I/O is explicitly unavailable.

## Public S1 prototype scope note

Source spot-review of the new public prototype confirms exact outer/leaf decoding, origin preservation, no structural retargeting from a DELTA origin to an older FULL anchor, and exact inode-leaf base validation. The new role shares `DeltaSearch` work/trial/fetch budgets with payload search. It can therefore change payload candidate outcomes through budget competition, even though the added eligibility is only inode leaves. Public results must disclose the now-mixed eligible counter population and measure total allocation; the offline S1 fixed-payload saving cannot simply be projected onto the public path. This is a scope qualification, not a request to silently split budgets or change the treatment.

Independent direction: prioritize **canonical-preserving actual-prior native-prefix encoding on existing chunks** as the larger production direction, while using the already promising minimal S1 public prototype to test structural origin delivery through the required smokes. Do not promote the whole-file rewrite merely because it was the initial architecture hypothesis. Final public acceptance remains contingent on the real smoke results and their exact identities, cleanup and allocation/timing boundaries.

## Public smoke applicability review before full157 results

Light-read `issue88-report-1/public-smokes.json` and committed `full-history-continuation.md` while the candidate full run was active. No artifact hashing, Store reads or other heavy analysis was performed concurrently. The six case rows belong to the three approved smoke selectors and cover33 verified mappings per arm. Correctness and cleanup pass; this is not final three-pair numerical qualification.

| Case | Baseline acknowledged allocation B | Candidate acknowledged allocation B | Baseline public operation sum ns | Candidate public operation sum ns |
|---|---:|---:|---:|---:|
|deepseek-five|3153920|3162112|1398389833|1277850917|
|sdk-binary-8m|8630272|8626176|31847668|38226958|
|fuse-binary-8m|9650176|9646080|207486042|236606540|
|fuse-text-32k|69632|69632|40463376|49133208|
|sdk-text-32k|77824|135168|26591584|39913794|
|small-files|208896|204800|20749209|25266709|

SDK-text aggregate public elapsed increases by about50.1%, and its return-to-A operation increases from about6.728ms to13.231ms. The reported extra matcher work and rejected alternative are real additional work, but do not explain the entire latency delta: the SDK edit preceding Commit also slows. No causal claim assigns all observed slowdown to structural encoding. Both SDK-text logical databases are77824 B with46 objects/62333 canonical B, so the57344 B allocated difference is a signed filesystem allocation adjustment, not57344 B of additional encoded metadata. One pair cannot establish its general allocation behavior.

The pre-execution continuation correctly retains every regression and distinguishes a full-history applicability experiment from accepting failed/tolerated historical gates. The21.9MB offline S1 opportunity concerned large full-history inode leaves; a fixed full157 public run can test whether it survives real admission, shared budgets and placement despite weak tiny-case results. Its baseline is historical, so no fresh paired timing claim is valid. The unchanged candidate identity, original limits and stop rules are recorded prospectively. Final full157 result acceptance remains pending; no release or134MB qualification follows from the smoke correctness PASS.
