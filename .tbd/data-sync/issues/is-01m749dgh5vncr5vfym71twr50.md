---
type: is
id: is-01m749dgh5vncr5vfym71twr50
title: "PR #209 (merge): final Fable design and technical review, then merge and prepare the release"
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
hold: null
hold_until: null
created_at: 2026-11-11T03:11:25.733Z
updated_at: 2026-11-11T18:15:50.283Z
started_at: 2026-11-11T03:11:32.406Z
---

## Notes

Reviews D-I done and dispositioned; CI green at 40f7adee; 0.6.0 release content is on the PR. Not merged: the Linux host gate (docs/project/guides/linux-host-gate.md) runs first on a real Linux host, then merge with the maintainer's confirmation in that session (github-merge: confirm-session), then release checklist steps 2-5, stopping before the tag. See the handoff comment on PR #209. Close at merge.
