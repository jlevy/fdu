---
type: is
id: is-01m3g3qyex5mxs5n2xnsh12wxn
title: Refresh macOS live-tool comparison with absolute throughput
kind: task
status: in_progress
priority: 1
version: 2
delegate: claude-code@spud10
labels:
  - performance
  - benchmark
  - macos
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-27T00:20:22.093Z
updated_at: 2026-09-27T00:20:36.673Z
started_at: 2026-09-27T00:20:36.672Z
---
Regenerate the claim-grade macOS tool-comparison matrix on the reproducible balanced 1M-entry corpus while the host is lightly loaded. Keep temporary builds and caches on /Volumes/spud-ext1, but create and measure the corpus, benchmark scratch, and results on the internal APFS volume. Include the available baselines (du, dumac, dust, diskus, dua, pdu, gdu, ncdu as valid) and update the main performance table with paired relative results plus absolute files/s and allocated GB/s throughput. Document the storage split and keep the benchmark process reproducible.
