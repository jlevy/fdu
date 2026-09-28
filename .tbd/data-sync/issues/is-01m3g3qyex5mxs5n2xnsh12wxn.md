---
type: is
id: is-01m3g3qyex5mxs5n2xnsh12wxn
title: Refresh macOS live-tool comparison with absolute throughput
kind: task
status: closed
priority: 1
version: 6
delegate: claude-code
labels:
  - performance
  - benchmark
  - macos
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-27T00:20:22.093Z
updated_at: 2026-09-28T16:20:45.275Z
started_at: 2026-09-27T00:20:36.672Z
closed_at: 2026-09-27T02:44:15.693Z
close_reason: "Implemented and published in PR #132. Refreshed the macOS comparison, added absolute throughput to the renderer and main table, documented internal measurement versus external build storage, committed raw evidence, passed full local make check, and all GitHub CI checks passed."
resolution: null
duplicate_of: null
---
Regenerate the claim-grade macOS tool-comparison matrix on the reproducible balanced 1M-entry corpus while the host is lightly loaded. Keep temporary builds and caches on /Volumes/spud-ext1, but create and measure the corpus, benchmark scratch, and results on the internal APFS volume. Include the available baselines (du, dumac, dust, diskus, dua, pdu, gdu, ncdu as valid) and update the main performance table with paired relative results plus absolute files/s and allocated GB/s throughput. Document the storage split and keep the benchmark process reproducible.

## Notes

Completed the macOS refresh at commit c5e2f6ed. The fixed matrix used the internal APFS volume for the generated balanced 1M corpus, benchmark temp/cache state, baseline, and results; build targets and uv/Cargo environments stayed on spud-ext1. Quiet mode was attempted but correctly rejected above the 25% CPU/load gate, so the completed matrix is explicitly uncontrolled. Nine competitors, 3 warmups/tool, and 12 adjacent timed pairs produced 0 invalid samples, 0 semantic mismatches, 0 oracle mismatches, no drift, and no mutation. fdu median: 5.991 s, 146,050 files/s, 0.499 allocated GB/s. Closest comparator: dumac +8.2% (95% interval +5.4% to +16.3%). README and benchmark runbook now show all baselines, absolute rates, work classes, and storage rules. make perf-test: 334 passed. Full make check passed after linking the ignored checkout target path to the task-specific external Cargo target so golden/package consumers used the verified build.
