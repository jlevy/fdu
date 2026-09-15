---
type: is
id: is-01kzs53e2agcaeebb7nypyp77g
title: "Cache retention policy: nothing prunes snapshots or bounds cache size"
kind: task
status: open
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-11T19:34:29.704Z
updated_at: 2026-09-15T22:27:24.059Z
---
Spec open question 5, unmapped until now. Nothing prunes snapshots for roots that are never queried again, and nothing bounds the derived-data layer's total size. A whole-drive scan writes a large snapshot; scanning many roots writes many, and they accumulate silently in the user's cache directory forever. Options are age-based GC, a size cap with LRU eviction, or manual-only via --cache-clear (already shipped). Needs a decision before the derived layer ships, because the derived layer multiplies the per-root cost. --cache-status and --cache-clear give the user the tools to do it by hand today, which is why this is a decision rather than an outage.

## Notes

2026-09-15 RECLAIM EXISTS, RETENTION STILL UNDECIDED (fdu-lh9x, PR https://github.com/jlevy/fdu/pull/67, commit 816fcf7):
The manual reclaim this bead relies on now covers every snapshot fdu wrote, not only ones the current build can read.
- --cache-status and --cache-status=all classify each file as current, stale (another format version, another engine fingerprint, or an unreadable header), unrecognized, or absent. They report sizes, and the text names the reclaim command.
- --cache-clear removes a root's snapshot, current or stale. --cache-clear=all removes every current and stale snapshot and reports the unrecognized files it left.
- Identification uses the snapshot magic plus, in a listing, the {16 hex}.fdu name. Symbolic links are never followed or removed.
- Before this, every release stranded every earlier snapshot, because engine_fingerprint mixes CARGO_PKG_VERSION.
Still open for this bead, unchanged:
- Nothing prunes automatically, and nothing bounds total cache size.
- No scope removes stale snapshots while keeping current ones. --cache-clear=all removes both; that narrower scope is fdu-m6lr.
- An orphaned .content sidecar, whose snapshot is gone, is listed as unrecognized and never removed. A later analyzed scan can still reuse it, so it is not stale by definition.
