---
type: is
id: is-01m2eegcapyzyrx7qngtzdhmtm
title: "PR #52 review PERF-7: one call tree quoted as two figures, with no profiling-binary identity"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-13T22:34:17.814Z
updated_at: 2026-09-13T23:09:10.680Z
closed_at: 2026-09-13T23:09:10.679Z
close_reason: "Fixed in a7d993f on PR #52: exp-102 and the parity plan cite the all-call-sites reading (1,993 of 2,391), recomputed from the committed trees. The profiling binaries' hashes were never recorded and the binaries were not retained, so an identity note beside each call tree records a null hash with the reason; re-capture with recorded hashes is appended to fdu-0q6w."
resolution: null
duplicate_of: null
---
PR #52 review PERF-7 (Low). docs/project/experiments/exp-102-point-lookup-for-public-mutation-preflight.md:386 vs the parity plan :1194 at afbb2ee. exp-102 quotes 1,993 of 2,391 (all call sites) and the plan quotes 1,936 of 2,358 (the largest site): two correct readings of one call tree, presented as one figure. The committed call trees carry no binary identity, so the 1a39be9 attribution cannot be checked from the artifact. Related to fdu-0q6w. Fix: cite one reading in both, and record the profiling binary's hash beside each call tree.
