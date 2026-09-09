# Prospective growth arithmetic — no new performance experiment

Owner steering: use short real-checkpoint subsets for iteration, full157 only for
final confirmation. Before experimentation, estimate growth from measured storage
components. This document records assumptions, not measured optimization gains.

Matched full157 growth: control 184,512,512 B; initial candidate 201,302,016 B.
Both start with 69,632 allocated bytes. Final allocations are 184,582,144 and
201,371,648 B. Candidate SmallContent FULL frames occupy 37,535,057 B and DELTA
frames 87,288,594 B (58,004 DELTAs). Small pack/record framing is 3,767,814 B;
large-file packs 8,120,302 B; metadata/legacy packs 49,441,790 B. SQLite/remaining
allocation contributes 15,148,459 B to growth. Metadata/legacy includes five old
chunk objects; it is not a claim that every v1 object is metadata.

Therefore, in bytes:

    optimized_growth = 114,013,422 + new_delta_frame_bytes
                       + change_in_FULL_frames + change_in_framing
                       + change_in_SQLite_and_allocation

This is exact component accounting. Holding the other components fixed is an
estimation assumption, not an exact prediction of filesystem page allocation.

Without extra FULL/overhead cost, tie/10%/20%/30% savings against control require
DELTA frames <=70.499/52.048/33.597/15.145 MB respectively, reductions of
19.23%/40.37%/61.51%/82.65%. With 8 MB extra FULL/overhead, these become
62.499/44.048/25.597/7.145 MB, reductions of 28.40%/49.54%/70.68%/91.81%.
Thus a 20% total-growth reduction needs 53.692 MB net removed from the current
candidate; merely making DELTAs 20% smaller barely reaches control parity.

Read-only post-proof diagnosis reconciles to the frozen SmallContent census:

| Actual target/base relation | DELTAs | DELTA frame bytes |
|---|---:|---:|
| Target more than 2x FULL base | 9,066 | 44,101,337 |
| FULL base < target <= 2x base | 34,988 | 36,145,399 |
| Target <= FULL base | 13,950 | 7,041,858 |

The largest anchor population is docs/config-catalog.md: base 49,124 raw bytes,
first original-oracle occurrence checkpoint 21; 87 dependents with 1,427,320
frame bytes. An actual target at checkpoint 122 has 127,653 raw bytes and a
25,992-byte DELTA frame. These occurrence dates establish fixture matches, not
an alternative candidate-selection experiment.

A worked hypothetical for that population: replacing three old-anchor DELTAs
with 45,000-byte FULL anchors and making the remaining 84 DELTAs average 4,000
bytes costs 3*45,000+84*4,000=471,000 bytes, versus 1,427,320 current DELTA bytes.
Net saving is 956,320 bytes before framing/page changes. The 45,000/4,000-byte
figures are assumptions, not newly measured encodings; extra FULL cost is included.

Conditional whole-workload scenarios for bounded FULL-anchor refresh:

| Scenario | Reduction in >2x / 1–2x / <=1x populations | Extra FULL + overhead | Estimated growth | Below control |
|---|---|---:|---:|---:|
| Conservative | 60% / 30% / 0% | 5 MB | 168.998 MB | 8.41% |
| Central | 75% / 50% / 0% | 8 MB | 158.153 MB | 14.29% |
| Optimistic | 85% / 65% / 20% | 10 MB | 148.913 MB | 19.29% |

Working estimate: roughly 158 MB retained growth, with illustrative 149–169 MB
sensitivity range. This is not a statistical confidence interval or proof that
those population reductions are achievable. It is a prospective model whose
unknowns the short real-checkpoint diagnostic must test. With the original fixed
base-selection policy unchanged, the measured expectation remains 201.302 MB.
Anchor refresh requires the owner's requested policy clarification; no such
product change or new performance run has been made for this calculation.

Evidence: /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-evidence/{control,initial}/census.json,
initial/anchor-diagnosis.json, growth-estimate.json, and the original performance
receipts/manifests. The completed same-Store verifiers passed both full157 arms.
