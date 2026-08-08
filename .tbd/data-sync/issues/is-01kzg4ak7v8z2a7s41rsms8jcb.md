---
type: is
id: is-01kzg4ak7v8z2a7s41rsms8jcb
title: "Revalidation: directory-mtime shortcut and parallel sweep streamed as deltas"
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:27:45.915Z
updated_at: 2026-08-08T07:28:38.441Z
---
The current revalidate() re-lists every directory and stats every entry. Make it the git-shaped tier it is meant to be:
- Directories whose own mtime is unchanged skip re-listing entirely (git's untracked-cache trick).
- Files with matching fingerprints keep their derived data (type verdicts, future content metrics) with zero reads.
- Only changed entries re-derive.
- Parallel sweep, not the current serial walk.
- Results stream as deltas so a caller serves the stale snapshot instantly and reconciles — and stale-while-revalidating is LABELED as such to consumers, never silently served as fresh.

Depends on the spike: if the sweep is not fast enough at 500k, this design changes.
