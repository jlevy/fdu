---
type: is
id: is-01kzysa7t79qpvyg6ec0zrdjbj
title: Document per-platform provenance for every scan tuning constant
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T00:03:56.359Z
updated_at: 2026-08-14T00:03:56.359Z
---
Every constant in scan.rs carries a doc comment citing the measurement that chose it, and every one of those measurements was on a 10-core M1 Pro on APFS. ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY is the clearest suspected mismatch: 30 microseconds was placed in the gap between APFS regimes of roughly 18, 22 and 42 microseconds per entry, but the Linux warm single-threaded floor is about 1.5 microseconds, some twenty times below the threshold, so the adaptive unlock may never fire on Linux. See docs/project/guides/platform-tuning.md.
