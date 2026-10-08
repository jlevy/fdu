---
type: is
id: is-01m4eax1x6y2vth78ghr8rxmtj
title: Attach a committed demo video as a verified release asset
kind: task
status: in_progress
priority: 1
version: 3
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
hold: null
hold_until: null
created_at: 2026-10-08T18:02:42.469Z
updated_at: 2026-10-08T18:02:47.596Z
started_at: 2026-10-08T18:02:47.593Z
---
A release commit that contains docs/media/fdu-demo.mp4 attaches it to its GitHub release as a twelfth asset, fdu-demo.mp4, digest-verified like the other eleven. A commit without it keeps exactly eleven, so v0.1.0..v0.3.0 still audit as eleven. The expected asset set is a deterministic function of the release commit's tree. Maintainer decision 2026-10-08. Scope: scripts/release/maintainer.py, scripts/release/announce.py, tests/release, docs/project/guides/release-process.md.
