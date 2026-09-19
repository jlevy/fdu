---
type: is
id: is-01m2xs3szwzwsyk5s25myn18hp
title: "H130: leftover after restore-without-classify"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels: []
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
created_at: 2026-09-19T21:28:16.635Z
updated_at: 2026-09-19T21:43:10.059Z
closed_at: 2026-09-19T21:43:10.059Z
close_reason: Confirmed exp-129. Restore classify gone after H129. path_of 11.85% of content_open. Snapshot 43.3%. No engine patch. Quiet 34.97%.
resolution: null
duplicate_of: null
---
H130 / exp-129 leftover after H129. Not a cut.

After restore-without-classify, a deciding-scale content-cache-hit leftover
profile names whether classify is gone from content_open and whether any
remaining userspace stage (path_of, HashMap insert, apply BTree, snapshot
load) is a new ≥3% wall mechanism that is not already rejected.

Not H116 (HashMap drop). Not file-count. Not threaded path_of unless the
sample names a skip that keeps completeness. Not a snapshot load on fdu PATH.

Determination: classify nodes <3% of content_open; named leftover is or is
not a new ≥3% userspace cut. No engine patch unless the sample names one.

Quiet once; if it fails, skip and label uncontrolled. Do not lower the 25% bar.
Attribution: H129 candidate counters-on hits plus a 20 s sample on the
profiling build. Optional same-source pair is attachment only.
