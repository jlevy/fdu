---
type: is
id: is-01m3n1xtgad2th9p6vn3t50y14
title: "H168: intern extensions without a per-file String"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/research/research-2026-09-28-pdu-and-the-linux-peer-gap.md
labels:
  - performance
  - linux
  - experiment
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
created_at: 2026-09-28T22:24:49.674Z
updated_at: 2026-09-28T22:24:49.674Z
---
classify::ext_bucket allocates a String per file before interning. Measure after the H157 rerun (fdu-o6um), which shares the allocation budget. Registry row in docs/project/guides/performance-loop.md; mechanism and pre-registration in the pdu brief.
