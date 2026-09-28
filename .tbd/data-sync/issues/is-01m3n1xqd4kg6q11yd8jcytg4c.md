---
type: is
id: is-01m3n1xqd4kg6q11yd8jcytg4c
title: "H164: classify .gitignore on walker threads"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/research/research-2026-09-28-pdu-and-the-linux-peer-gap.md
labels:
  - performance
  - linux
  - experiment
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
created_at: 2026-09-28T22:24:46.500Z
updated_at: 2026-09-28T22:24:46.500Z
---
After H162-H163 classification is still serial on one consumer (~100 ms above the --no-controls walk on linux-v6.12; default tree 211 ms vs pdu 70 ms). Carry each directory's governing chain and ignored state down the walk queue so walkers classify their own listings in parallel. Registry row in docs/project/guides/performance-loop.md; mechanism and pre-registration in the pdu brief.
