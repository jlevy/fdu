---
type: is
id: is-01m3n1xrwahrnxyrsczz3vhc23
title: "H166: work-stealing walk queue instead of Mutex+Condvar+notify_all"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-28-pdu-and-the-linux-peer-gap.md
labels:
  - performance
  - linux
  - experiment
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
created_at: 2026-09-28T22:24:48.010Z
updated_at: 2026-09-28T22:24:48.010Z
---
fdu walkers take 14.7k voluntary switches per 1M entries vs pdu's 17 (rayon). Worker-local deques with stealing or a lock-free queue. Registry row in docs/project/guides/performance-loop.md; mechanism and pre-registration in the pdu brief.
