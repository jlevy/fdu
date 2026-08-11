---
type: is
id: is-01kzsb3q4wbps8tsndgpg6wjwd
title: "PR#6 R5: from_run drops interval-derived flags for pre-split runs"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:19:30.459Z
updated_at: 2026-08-11T21:19:30.459Z
---
benchmarks/realtree/experiment.py:321-326. ci_excludes_zero=False and direction='unknown' are defaulted rather than derived from ci95 bounds, so a recorded artifact contradicts its own interval for pre-split runs. Medium.
