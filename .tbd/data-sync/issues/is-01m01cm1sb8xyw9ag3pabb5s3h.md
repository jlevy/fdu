---
type: is
id: is-01m01cm1sb8xyw9ag3pabb5s3h
title: Stabilize adaptive scan scaling on heterogeneous macOS trees
kind: bug
status: open
priority: 1
version: 3
labels:
  - perf
  - macos
dependencies: []
created_at: 2026-08-15T00:19:49.615Z
updated_at: 2026-08-15T00:27:34.014Z
---

## Notes

Observed with installed fdu 0.1.0-dev+ge53f70802.dirty on /Users/levy/Library/Application Support (live, partial tree; about 396,900 entries and 12 macOS TCC permission errors). User sample: dust 2.284 s versus fdu 4.008 s. Total CPU was nearly equal (dust 11.797 s, fdu 11.515 s), but CPU/wall was about 5.17 for dust versus 2.87 for fdu, pointing to under-parallelization rather than extra filesystem work. Fixed-thread perf_probe diagnosis showed automatic mode was bimodal on the same tree: two runs stayed at exactly 6 effective workers and took 2.27-2.35 s; two crossed calibration and expanded to about 15 workers, taking 1.62-1.75 s. A four-pair auto-vs-16 diagnostic had a -6.85% paired median for fixed 16 but is not claim-grade because the tree is live and partial. The 30,000 ns/entry early threshold is near this heterogeneous APFS workload's boundary and makes scaling depend on which directories/cache state the first 16,384 entries sample. Separately, dust --print-errors and fdu reported the same 12 protected directories; dust exited 0, fdu correctly exited 2 unless --allow-partial was supplied.
