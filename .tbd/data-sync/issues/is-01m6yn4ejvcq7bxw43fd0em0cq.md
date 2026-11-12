---
type: is
id: is-01m6yn4ejvcq7bxw43fd0em0cq
title: "Ship recent growth: CHANGELOG, make check, PR with validation plan, reviews"
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-08T22:40:45.146Z
updated_at: 2026-11-12T00:56:36.165Z
closed_at: 2026-11-12T00:56:36.165Z
close_reason: "PR #209 merged as 8a8febc7 (tree 64a7d366, the gated tree) after reviews D through L, the Linux host gate at 98784661, and CI green at that head. 8a8febc7 is the 0.6.0 release commit."
resolution: null
duplicate_of: null
---
CHANGELOG [Unreleased] for change time, coverage note, ages, volumes, schema fdu.report/12, public API additions and the report signature change; make check; draft PR via create-or-update-pr-with-validation-plan; senior review plus a correctness review of window and band logic and a security review of platform capacity code; address findings.

## Notes

Open until merge. PR #209 is ready for review at 40f7adee with the 0.6.0 CHANGELOG, version bump, and release notes. Remaining: the Linux host gate, then merge (see fdu-gp38 and the PR handoff comment).
