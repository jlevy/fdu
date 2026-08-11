---
type: is
id: is-01kzs53e2agcaeebb7nypyp77g
title: "Cache retention policy: nothing prunes snapshots or bounds cache size"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
created_at: 2026-08-11T19:34:29.704Z
updated_at: 2026-08-11T19:34:29.704Z
---
Spec open question 5, unmapped until now. Nothing prunes snapshots for roots that are never queried again, and nothing bounds the derived-data layer's total size. A whole-drive scan writes a large snapshot; scanning many roots writes many, and they accumulate silently in the user's cache directory forever. Options are age-based GC, a size cap with LRU eviction, or manual-only via --cache-clear (already shipped). Needs a decision before the derived layer ships, because the derived layer multiplies the per-root cost. --cache-status and --cache-clear give the user the tools to do it by hand today, which is why this is a decision rather than an outage.
