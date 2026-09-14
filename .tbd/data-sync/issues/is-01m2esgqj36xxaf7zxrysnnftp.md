---
type: is
id: is-01m2esgqj36xxaf7zxrysnnftp
title: Measure the allocation-slope ceiling for the controls-enabled detached route per platform
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-14T01:46:43.650Z
updated_at: 2026-09-14T01:46:43.650Z
---
Residual of PR #52 review PERF-6 (fixed as fdu-b6oe in ee5fe1c), recorded by the fixer.

**What landed.** A `cfg(feature = "gitignore")` twin inside `construction_routes_keep_their_allocation_and_work_boundaries` (`crates/fdu/tests/detached_performance_invariants.rs`). It asserts the detached route counters for a `.gitignore` fixture, counts the control as one accepted op, and checks ignore classification. A controls-enabled detached scan that silently fell back to the streaming reducer now fails a test.

**What did not land.** No allocation-slope ceiling for the controls-enabled detached route, meaning allocations per entry, reallocations, and bytes as the tree grows. fdu-b6oe's close reason: "No allocation slope: that ceiling needs a per-platform measurement." The controls-off route has its allocation guard. The controls-enabled route (default features, `open`, `--watch`, and the `cold-scan-index` probe) has route counters only, so a per-entry allocation regression on that route passes every deterministic check.

**To do.**
1. Measure the controls-enabled detached route's allocation slope on Linux, macOS, and Windows (CI runners are enough for a deterministic counter slope). Use a control-rich fixture at two or more sizes.
2. Set a ceiling with headroom per platform, or one ceiling if the platforms agree. Record the measured values and regime.
3. Add the ceiling to the existing test, with a negative test showing it catches an injected per-entry allocation.

Relates to fdu-lj4h, which owns the negative-tested per-entry allocation guards for parity.

Review: https://github.com/jlevy/fdu/pull/52#pullrequestreview-5192264318
