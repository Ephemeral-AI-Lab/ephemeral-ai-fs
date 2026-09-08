# Matched offline read-cost follow-up v1

Frozen after encoding and before readmeasurement. No newencoding/parameter/sample
population. Read existingsealedS2/S3 framedimages withsame decoder on selected
firstordinal,largestpayload,andmaximumclosure record(tieslowestordinal).
For eachrecordcompareitsFULLcontrol againstselectedrepresentation forsmallrange
firstmin4096bytes andfullunit. Threealternatingpairedreads C/B,B/C,C/B; eachfresh
codeccontext/noapplicationpayloadcache, OS cacheuncontrolled (notdiskcold). Hashall
returned/reconstructedbytes. Warmrepeat explicitlyonone-unitPythondecodedcache,
notrealpublicread. Preserveallrawns/counts, reportmedian/min/max descriptively.

Thisfills the missingmatchedFULLread comparator; itdoesnotchange orresamplethe
originalbenchmarks. Inputfile/manifests hashverified before/after, no framewrites.
Resourcewall900s, underexistingmeasurementlock, no parallelbuild/benchmark.
Outputsnewissue88-read-costs-1, fixed1run, noadaptiveadditionalrepetitions.
