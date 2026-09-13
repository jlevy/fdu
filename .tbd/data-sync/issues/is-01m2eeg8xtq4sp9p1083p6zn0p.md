---
type: is
id: is-01m2eeg8xtq4sp9p1083p6zn0p
title: "PR #52 review PERF-6: the detached route guard covers only read_controls false"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-13T22:34:14.329Z
updated_at: 2026-09-13T23:00:22.096Z
closed_at: 2026-09-13T23:00:17.505Z
close_reason: "Fixed in the PR #52 test commit (see notes): a cfg(feature = gitignore) twin inside construction_routes_keep_their_allocation_and_work_boundaries asserts the detached route counters for a .gitignore fixture, counting the control as one accepted op, and checks ignore classification. No allocation slope: that ceiling needs a per-platform measurement."
resolution: null
duplicate_of: null
---
PR #52 review PERF-6 (Low). crates/fdu/tests/detached_performance_invariants.rs:81 at afbb2ee. The deterministic route and allocation guard measures only read_controls: false. The controls-enabled detached route has digest-equality tests (scan.rs:5391-5440), which would pass just as well if that route silently went back through the streaming reducer. Fix: add a cfg(feature = gitignore) twin with a .gitignore fixture asserting the same route counters. The test binary measures process-wide counters, so the twin runs inside the existing test rather than as a concurrent one.

## Notes

Fixed in ee5fe1c on PR #52.
