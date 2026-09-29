---
type: is
id: is-01m3n1xrwahrnxyrsczz3vhc23
title: "H166: work-stealing walk queue instead of Mutex+Condvar+notify_all"
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
created_at: 2026-09-28T22:24:48.010Z
updated_at: 2026-09-29T11:36:09.965Z
closed_at: 2026-09-29T11:36:09.965Z
close_reason: "Closed by its pre-registered gate (overnight plan amendment 8), no build: perf_probe scan-index --threads 4 on the H180 head, FDU_COUNTERS=1 (proportions only): walker starved_ns 2.2% of walker time on linux-v6.12 and 0.6% on node-modules-dense, lock_wait ~0. Walkers do not starve; H181 (fdu-uk0u) takes the separate futex-syscall cost. Note: send_ns was 11% of walker time on node-modules-dense under counters (allocation-heavy recycle path), worth an uninstrumented look."
resolution: null
duplicate_of: null
---
fdu walkers take 14.7k voluntary switches per 1M entries vs pdu's 17 (rayon). Worker-local deques with stealing or a lock-free queue. Registry row in docs/project/guides/performance-loop.md; mechanism and pre-registration in the pdu brief.
