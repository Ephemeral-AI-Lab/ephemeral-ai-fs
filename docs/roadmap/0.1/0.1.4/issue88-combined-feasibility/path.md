# Bounded path to the combined encoded milestone

I read the updated #88 body and all four comments through `gh api` (ordinary
`gh issue view` failed on GitHub's deprecated projectCards field). Source is
issue88 checkout76b2443270e1ace1b9d6cb8c5e3fb587e55b64d6. No Store, scanner,
replay, build, encoding or product edit was performed. Earlier source-flow
references remain current. This document is prospective sequencing only.

## Decision

Pursue **existing canonical chunks + native actual-prior payload encoding + fixed
S1 structural-origin encoding**. Run the finite delivery diagnostic once as a
prerequisite, while preparing the native format/reader/writer design in parallel.
Do not wait for diagnostic results to begin code-interface/specification review,
and do not end the project at another diagnostic report. The diagnostic selects
whether a separate coverage change is needed before the native treatment; it is
not a requirement to explain every byte before implementation can begin.

159163199 bytes =102306097 historical S2 framed-content bytes +56857102 historical
S1 group bytes is a **component milestone to test**, not a measured combined
image, lower bound, or complete Store forecast. It mixes independent physical
encodings and excludes additional costs. The134221004-byte complete allocation
stretch remains separate. The milestone permits a realistic first integration
without needing an invented70MB payload target.

## Public-delivery bridge already present, and what still differs

The public producer already resolves an old regular-file root from base inode
identity or same-path base namespace lookup (`changes.rs:1075–1182`) and attaches
it at`:1461`. File payload construction supplies first spans (`objects.rs:2713`),
and private spill preserves them. These existing paths are sufficient in
principle to find chronological same-file chunk bases without importing offline
source-history manifests into the product.

What is not equivalent to offline S2:

- Public hints are computed in `objects.rs:2047–2073` before admission's initial
  CAS probe (`objects.rs:2978`). They consume131136-byte grants from1MiB/file
  and16MiB/operation caps:7 and127 metadata reads respectively. Existing evidence
  shows152 full157 Commits saturating the latter. S2 computes complete same-path
  overlap after global CAS and does not pay that public correspondence cap.
- Offline first occurrence follows checkpoint/path/chunk order. Public first
  occurrence follows dirty-task/producer/queue progress, and existing dedup
  keeps its first hint state. Same canonical chunks can have different candidate
  predecessors in different paths. Do not rewrite scheduling to emulate offline
  order as an undeclared part of the native codec treatment.
- Public hints contain up to four overlaps. S2 selects the first overlap. The
  candidate rule for the native treatment must be explicit; retaining only the
  first supplied ID is a declared treatment policy, not unchanged search.
- `read.rs:219–283` returns an anchor reference for an existing DELTA rather
  than its reconstructed bytes. `admission.rs:492–496` then loads that FULL
  anchor. Native S2 uses the actual previous target. Reader support must expose
  bounded authenticated actual-prior reconstruction, not quietly keep anchor
  substitution and claim the S2 treatment was implemented.
- `pack.rs:100–132` currently accepts FULL and COPY/INSERT DELTA records only.
  `read.rs:516` requires a FULL base. Native-prefix chains therefore need an
  explicit versioned record and reader graph, not a compression-level switch.

On current S1, inode leaves have origin hints with no file span intentionally
(`objects.rs:2720–2729`). Exact file-payload/source eligibility is mandatory.
Metadata-value chunks are also outside the required file-span population.

## Concrete stages and exit gates

| Stage | Control and one declared variable | Required decision / stop boundary |
|---|---|---|
|D: bounded delivery diagnostic|Accepted M4.5 `eb7050603`; instrumentation only|One complete valid post-CAS file-payload count/byte cohort and causal outcomes. If a transport/accounting bug invalidates it, fix that demonstrated defect and repeat only the affected diagnostic under a new identity; do not start broad new data collection.|
|C: conditional coverage treatment|Same accepted M4.5 policy plus the single demonstrated coverage change, only if D establishes that delivery blocks useful byte-bearing targets|Missing required span means repair that exact handoff. True limiting policy means declare one bounded scheduling/allowance change and its resource cap prospectively. No codec, chains or unrelated metadata change. Retain or reject using added delivered candidate bytes and foreground/read work; reused-triggered grants alone are not avoidable work. If existing delivery is sufficient for a useful P test, skip C.|
|P: native physical payload implementation|Freeze C if accepted, otherwise B=M4.5. Add one cohesive native-prefix physical representation/base policy; canonical units and structures unchanged|Reader/writer integrity and deterministic component accounting first; approved smokes before public full157. FULL fallback is normal for absent/inadmissible/nonwinning bases. Malformed identity/frame/graph is an error, never a quiet FULL retry.|
|SP: combination|Hold exactly the approved S1 source behavior fixed; add exactly P on the same accepted B/C foundation|Sequential matched S1-only versus S1+P establishes marginal payload treatment on fixed structure. Include P-only evidence on the same foundation to reveal structural/payload interaction. Measure a combined representation and then complete acknowledgement allocation, not copied component figures.|

The P treatment is one declared physical representation change; native codec,
actual-prior reconstruction, its record kind and dependency bounds are inseparable
implementation requirements, not a claim that a single compression knob changed.
Do not simultaneously tune its parameters. Starting settings can reuse S2's
Zstd1.5.7/level3/window20/checksum/content-size/single-thread, first chronological
actual-prior ID,4 edges/1MiB complete decoded closure and strict whole-record win.
Whether production groups store independent prefix records or prefix-specific
groups must be frozen before comparisons: copying the experimental56-byte
header uncritically into a different pack format is not required or justified.

For S2-sized units<=32768 bytes,4 edges mean at most163840 reconstructed raw
bytes in a simple five-object chain; the1MiB cap remains an additional format
bound. Measure actual cold-application-cache closure reads and memory anyway;
bounded decoded bytes do not prove acceptable latency.

## Minimal ownership split while D is pending

1. **Delivery/diagnostic owner:** `objects.rs` handoff and `objects/spill.rs`
   transport, with fixed aggregate receipt fields. Reuse the two explicit tag/
   grant bytes, prove actual layout/queue/group budget equivalence, account
   duplicate credit once, filter file payloads exactly. No global ID registry.
   The diagnostic's eligibility denominator is after initial CAS and before
   optional matcher-memory gates; carry final outcome through publication.
2. **Physical representation owner:** `objects/pack.rs` format/codec validation
   and `objects/read.rs` bounded actual-prior reconstruction. Specify magic/
   version dispatch, authenticated base ID/output length, depth/cumulative-work
   validation and canonical reconstruction. Existing legacy FULL/DELTA read
   behavior stays strict; no in-place migration. Reuse installed Zstd rather than
   a second delta language or backend. Design and source review can proceed
   immediately; execution remains serialized under the frozen contract.
3. **Admission/integration owner:** `objects/admission.rs`, current hint selection
   and prepared/publish accounting. Apply P only to exact file-payload roles;
   keep S1 structural-origin FULL-base rules unchanged. Count attempted versus
   durable winners separately; serialize all dependencies with the public
   acknowledgement. No offline candidate manifest lookup.
4. **Independent verification owner:** combined account and controls, role/group
   attribution, canonical/retained-root/dependency validation, read probes and
   final pre-verification snapshot custody. Reuse the shared final inventory.

Files shared between owners need explicit method boundaries or sequential edits;
“parallel work” does not mean concurrent edits to `objects.rs`. Builds, encoding,
smokes and replay remain serialized. Source/design/review are the useful parallel
work that avoids turning D into a schedule-wide pause.

## Diagnostic completion gate, without a perpetual loop

Required outputs are fixed: file-payload eligible count/canonical bytes; six
exclusive cursor states; first/inherited memory/file/operation/descriptor limits;
grants triggered by eventual preexisting/missing/duplicate occurrence; and normal
candidate/base/terminal outcomes. Require a source-proven attempt-to-selected
bijection, zero late races for the simple unique-cohort claim, and successful
transaction pack-range provenance joined once to final locators. No per-Commit
Store traversal or per-object timed logging.

D is complete when those declared fields and gates answer which public barrier
exists. It need not predict final compressed savings: S2 already demonstrates
potential, and P will measure durable realization. Completed no-overlap, absent
predecessor and nonwinning prefix are legitimate FULL outcomes. A partial hint
with an eventual DELTA is not a budget failure. If a valid D still leaves uncertain
net benefit, proceed with the bounded P experiment under existing coverage and
report that uncertainty rather than commission another general diagnostic.

The offline candidate class may be implemented as an explicit public policy
component: use the producer's already captured prior FileState root and span to
look up the first overlap at the declared CAS boundary. That is a legitimate
candidate-delivery treatment C, not a dependence on offline evidence. However,
it must be tested separately before P when it changes coverage or ownership, as
the issue requires. **Bit-identical reproduction of S2's global path-ordered
manifest is neither required nor a good product interface.** Do not introduce a
source-history manifest, full global sort or global search merely to reproduce
an offline frame total. Public first-owner identity and explicit limits may yield
different bases and a larger candidate image.

The concrete default after valid D is P using the first currently delivered
payload hint and actual-prior reconstruction. Choose one C first only when D
establishes a specific material public delivery obstacle and a bounded fix is
justified. If C fails its prospective test, retain that failure and either run P
with existing limited delivery or reject the milestone's feasibility under the
declared costs. Do not restart D, silently raise allowances, or require another
general research cycle merely because159163199 was not attained.

## Comparable controls and interaction accounting

Label the foundation B (accepted M4.5), optional C (accepted coverage policy),
S (fixed S1) and P (native payload). The meaningful small matrix is
foundation, foundation+S, foundation+P, foundation+S+P. Compare one factor per
edge. Reuse existing B/S controls only where producer content seals, workload,
environment, framing and timing boundaries truly match. Historical receipts can
be descriptive storage controls; they are not fresh timing pairs. If C changed
the foundation, earlier B+S is no longer the matched control for C+S+P.

S1 and payload candidate search currently share admission/batch resources.
Their interaction can change which candidates fit, CPU cost, group formation
and persisted pack coverage even if logical payload/structural groups are
separate. Report actual payload frame/group bytes, structural groups, shared/
mixed classifications, headers, locator indexes and required base closure. Do
not allocate compressed mixed bytes proportionally or count a shared base twice.
Both successful and rejected arms must retain their negative read/timing results.

Freeze smoke and full-history thresholds/resource boundaries before each run.
Pass correctness and cleanup first; judge approximately30% guidance together
with absolute small-call costs, not a newly invented acceptance threshold. Final
acknowledgement allocation is measured on the original fresh Store; snapshot/copy
allocation cannot replace it. Close the milestone only on measured combined
representation evidence; do not close release/allocation goals just because this
plan or its diagnostic is complete.
