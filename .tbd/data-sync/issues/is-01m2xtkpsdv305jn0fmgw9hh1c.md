---
type: is
id: is-01m2xtkpsdv305jn0fmgw9hh1c
title: "H132: leftover after restore parent-path join"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
created_at: 2026-09-19T21:54:26.220Z
updated_at: 2026-09-19T22:06:34.635Z
closed_at: 2026-09-19T22:06:34.634Z
close_reason: "exp-131: confirmed leftover; restore-walk path_of gone; snapshot path_of 9.89% of content_open discarded on one-shot serving=None; no engine patch"
resolution: null
duplicate_of: null
---
H132 / exp-131. Leftover profile after H131. Not a cut unless the sample names a skippable >=3% userspace mechanism that keeps completeness and path identity.

Determination: path_of is gone from content_open (<3%), and the named remaining leftover is or is not a new >=3% userspace cut (join, commit_record, snapshot, HashMap). Do not retry H116. Do not mint a snapshot-parse cut. Do not retry H131.

Quiet once; if fail, skip and label uncontrolled. Same-source pair is attachment. Attribution: 20 s sample on the profiling build plus counters-on hits.
