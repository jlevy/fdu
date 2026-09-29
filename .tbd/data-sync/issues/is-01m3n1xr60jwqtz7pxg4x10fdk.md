---
type: is
id: is-01m3n1xr60jwqtz7pxg4x10fdk
title: "H165: revisit the Linux walker count after H162-H163"
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/research/research-2026-09-28-pdu-and-the-linux-peer-gap.md
labels:
  - performance
  - linux
  - experiment
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
created_at: 2026-09-28T22:24:47.296Z
updated_at: 2026-09-29T11:16:21.383Z
closed_at: 2026-09-29T11:16:21.382Z
close_reason: "Screened in exp-182 on the H172 head: 3 walkers regress (+10.25%, +21.95%), 6 and 8 do not clear on both real subjects; PORTABLE unchanged."
resolution: null
duplicate_of: null
---
Probe sweep on linux-balanced-1m: default-tree 1.42 s at 4 walkers, 1.28 s at 8; H84's /usr regression came from the gitignore-on default that H162-H163 cut. Not a retry of H84. Registry row in docs/project/guides/performance-loop.md; mechanism and pre-registration in the pdu brief.
