# Final combined resource/tradeoff review

Reviewed existing JSON/CSV/source only. No rerun, hash scan, Store/inventory open, build, census or encoding. Sources: issue88-SP-resources-1/resource-timing-summary.json and both checkpoint CSVs; issue88-SP-report-1/comparison.json and findings-and-next-experiment.md. Resource report statusPASS, invalid_counter_or_residual=false; both CSVs contain315 rows (158performance+157verification), with no invalid field statuses. Both normal verification/cleanup gates nowPASS. Large artifact authenticity is the upstream sealed proof's responsibility, not re-established by this review.

## Finding requiring final prose correction

The draft accurately reports performance construction improvements, Commit write-I/O growth, Exec CPU growth and the earlier depth0 read-probe regressions. It should additionally disclose the **new full157 historical verification read-and-digest regression**. This is a material measured cost now available, not an unmeasured hypothetical:

|Historical verification Exec quantity|Control|Candidate|Candidate minus control|
|---|---:|---:|---:|
|Elapsed ns|395368430006|487045982584|+91677552578 (+23.1879%)|
|Host CPU ns|169329383787|213648809324|+44319425537 (+26.1735%)|
|Host sampled current RSS max bytes|74235904|78233600|+3997696 (+5.3851%)|
|Host recorded read-I/O bytes|346378240|235192320|-111185920|

These executions perform complete historical filesystem observation/read/hash, including their fixed public process/transport path. They are verification-mode costs, not pure random-read latency, not codec-only time, not OS-cold reads and not contamination of construction performance. Lower read-I/O observations alongside higher host CPU are consistent with a representation/decoding cost tradeoff, but source/aggregate data alone do not prove that chains account for the entire slowdown. The separate intervening build/cache/environment history remains disclosed. Present one sequential pair, no repeatability claim.

Verification enclosing invocation is532819404667→618709495750ns (+85890091083ns), including its separate preparation/setup/observer scopes. Keep this separate from verification Exec. Full performance+verification invocation totals, if quoted only as a descriptive campaign cost, are1095520110376→1110990823750ns (+15470713374ns,+1.4122%); this does not include every external census/build/wait and must not replace a public-operation metric or support an overall product speedup claim. No need to headline this aggregate; the specific verification row above is clearer.

## Exact performance tradeoffs checked

|Quantity|Control|Candidate|Difference|
|---|---:|---:|---:|
|Public Exec ns|245932122164|234598445543|-11333676621 (-4.6085%)|
|Public Commit ns|79272777754|47805663491|-31467114263 (-39.6947%)|
|Public Exec+Commit ns|325204899918|282404109034|-42800790884 (-13.1612%)|
|Exec host CPU ns|54031968422|56935452885|+2903484463 (+5.3736%)|
|Commit host CPU ns|78462690590|48298782291|-30163908299 (-38.4436%)|
|Commit host write-I/O bytes|1933914112|2025394176|+91480064 (+4.7303%)|
|Max sampled current host RSS across reported performance phases bytes|117473280|114245632|-3227648|
|Performance host runtime allocated sample maximum bytes|218771456|185253888|-33517568|

Exec+Commit host CPU132494659012→105234235176ns (-27260423836ns,-20.5747%). This is phase-local process-counter aggregation, not total host CPU for the whole campaign. Commit traffic increases despite smaller allocation; no claim of universal write-efficiency improvement is justified. Host process I/O counters and requested native BLOB bytes are different dimensions.

Performance container CPU between the initial work-boundary observation and last checkpoint is118098626000→107810051000ns. Verification's analogous container interval is150698897000→169958804000ns. These include control/observer activity between boundary samples; they are not exact public-call CPU. Last-observed lifetime cgroup memory peaks are performance165748736→150663168B and verification267423744→258347008B. These are cgroup total-memory peaks including file/kernel categories, not process RSS or phase-local incremental peaks. The last observation precedes final close/container cleanup.

Spool allocated maximum647168B and container staging allocated maximum70705152B are unchanged in performance. Host-runtime disk already includes Store and spool. Do not add its185253888B to the184598528B Store or647168B spool. Free-disk columns remain null in all315rows/arm because the normal wrapper checks a threshold without retaining each observed byte value. Container process RSS and block io.stat counters are also correctly unavailable rather than zero. The one schedule-level free-byte observation is not a minimum over the campaign.

## Timing scope verification

Performance preparation73975454250→68988315583ns is outside case/public timing. Case work474087771333→421075015916ns includes per-step public work, transfer and outside-step observations. Sum of step walls419869718837→371689301001ns excludes the subsequent cgroup/disk/staging observations and step artifact persistence. Their difference54218052496→49385714915ns is an unattributed observation/control interval, not measured pure observer time. Public Init/fork/mount are nested in case setup; public End is in close. Neither is additive to those enclosing timers.

Snapshot observer966078500→939473500ns and census22222320416→19107231333ns are explicit separately bound observer durations. They are outside performance public elapsed. They do not substitute for unrecorded validation/report observer durations. The final resource script rechecks small source receipt/custody/schedule files before sealing, validates completed container removal, and propagates invalid residuals/counters toINVALID. I found no resource arithmetic/custody-status failure in the generated report.

## Decision and exactly one next action

I agree with the final draft's selected **unchanged-policy actual retained depth0–4 public-read qualification** as the one next action. My earlier root-anchor substitution proposal remains a plausible future optimization hypothesis, but it is not selected now:88.8MB depth-fallback canonical bytes have no exact durableFULLframe split, current actual depth4 records are numerous, earlier microprobes covered onlydepth0, and the now measured whole-history read-and-digest CPU/elapsed regression makes qualifying current read-tail behavior the stronger immediate decision.

This does not block completion of the implemented research milestone: both current arms verified and the candidate retains184598528B acknowledged allocation. Keep the implementation as an isolated research checkpoint, record all read/write/CPU tradeoffs, and do not extend depth, tune anchors, rerun for better numbers or chase134.2MB under this report. Normal full157 verification is now complete; the prospective read diagnostic is separately authorized follow-on work, not a replacement verification or a claim that this report is unfinished.
