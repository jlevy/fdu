---
type: is
id: is-01m4xn8gwp26wpnjtp7jnghfh0
title: Audit native lifecycle policy and run full handoff gate
kind: task
status: closed
priority: 2
version: 5
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T16:52:20.500Z
updated_at: 2026-10-14T17:59:33.580Z
started_at: 2026-10-14T16:52:53.661Z
closed_at: 2026-10-14T17:59:33.580Z
close_reason: "Completed in PR #205 at bd7de58de9275919f740145a9d54926bd5f7a6e6: reconciled architecture and API guidance, eight-item tracked roadmap, three focused runtime fixes, and independent Astra review with no findings. All required local make check stages passed after a scoped embedded-Python host setup correction; 68 targeted and 81 full Python tests passed. CI run 38609685963 passed all 20 jobs on this exact commit. Eight future improvement beads remain open; no new performance claims."
resolution: null
duplicate_of: null
---

## Notes

Implemented and independently reviewed three focused fixes: unsupported opened projections raise before native dispatch; native total/rollup include provenance under one borrow; typed Index.freshness avoids report construction in adapter. Added deterministic regression tests and corrected self-contained README fence. Targeted suite passes 68 tests after replacing an invalid root-provenance timestamp assumption with a direct no-second-read guard; removal case demonstrates mixed-state failure. Astra final review has no findings. Full make check running on final source with external worktree-owned target/env/cache paths; formatter/structural checks/clippy pass so far. Existing GIL follow-up includes implicit last-reference teardown; added optional cached-index verification roadmap bead fdu-l9jh. No performance measurements or claims.
