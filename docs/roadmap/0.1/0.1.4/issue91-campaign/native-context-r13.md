# R13 prospective native context reuse

G6 (product e218653f4; clean build501fe5a5c) records Init1000005,417,598,166ns
and creation5003,845,742,042ns with four passing independent proofs. Both remain
unqualified against the published baseline. SQL/admission repairs are material,
but creation still carries native encoder work and the4KiB publication costs.

R1 reused the1MiB backing allocation, but native_compress_in still calls
ZSTD_initStaticCCtx for every frame, rebuilding workspace/context state. Existing
profiles show codec context initialization inside native encoding. Reuse the
initialized static context only while its exact owned workspace pointer and length
remain unchanged. Keep the identical setter sequence, frame flags/level, prefix
binding, end-of-call reset, encode output bounds, and release before predecessor
reconstruction. Never keep a borrowed prefix/input or a global encoder. On a changed
workspace or failed initialization discard the cached context; preserve error
recovery. No dependency code or codec version changes.

Strengthen the existing R1 byte-equivalence/resource/error test with a test-only
count of actual static-context initialization calls: stable backing must initialize
once, not on each ordinary frame. Retain its before failure. Reused frames must
remain byte-identical to both fresh static and frozen dynamic controls across the
existing size/prefix boundaries, alternating prefixes and error recovery. A changed
frame or resource contract rejects this treatment. Additional physical-format,
canonical, collision/rollback and bounds tests remain required.

If validated, freeze G7/source and distinct isolated binary/image identities, then
repeat the R6 four cells/order/seed1 and their independent proofs once. This does
not reset any acceptance threshold or replace earlier valid samples. Storage
improvements,4KiB creation, bounded admission, full157 and every affected final
matrix/independent review obligation remain binding.
