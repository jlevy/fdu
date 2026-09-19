---
type: is
id: is-01m2xptsdmc6vtbekr1xtwfjgh
title: "H126: post-H125 cache-hit leftover profile"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
delegate: unknown@spud10
labels:
  - performance
  - campaign-2
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
hold: null
hold_until: null
created_at: 2026-09-19T20:48:23.984Z
updated_at: 2026-09-19T20:56:18.746Z
started_at: 2026-09-19T20:48:27.800Z
closed_at: 2026-09-19T20:56:18.744Z
close_reason: "H126 confirmed (exp-125): completeness walk gone (0.007% of content_open). First analysis_candidates walk remains 15.7%. Restore mix unchanged. No new userspace cut. Do not retry H116."
resolution: null
duplicate_of: null
---
Pre-registered 2026-09-19 ~13:48 PT.

H126 / exp-125. After H125, the second completeness walk should be gone.
This is a leftover profile, not a cut.

Determination: on deciding-scale frozen metabrowser-clone content-cache-hit,
the open_for_report completeness walk is under 3% of content_open (or absent),
and the named remaining leftover is or is not a userspace stage at least 3%
that is not already rejected (H116 candidates HashMap, H112 parse, H109
controls).

Control = HEAD with H125 in (be8d4d69 / 25f423fd). Same-binary pair.
FDU_COUNTERS unset on the pair. Attribution: counters-on hits + 20s sample.
Uncontrolled allowed. Do not lower the 25% bar.
Do not compile a new engine cut unless the sample names one at the 3% bar.
