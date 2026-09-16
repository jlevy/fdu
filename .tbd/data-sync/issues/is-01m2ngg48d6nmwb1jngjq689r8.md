---
type: is
id: is-01m2ngg48d6nmwb1jngjq689r8
title: "PR #64 review RN64-5: 'No snapshot written by an earlier build serves 0.1.0' overstates it"
kind: bug
status: closed
priority: 3
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2ngfd0y2yzwg2v10p2j601z
hold: null
hold_until: null
created_at: 2026-09-16T16:23:47.724Z
updated_at: 2026-09-16T16:34:19.453Z
started_at: 2026-09-16T16:25:04.055Z
closed_at: 2026-09-16T16:34:19.452Z
close_reason: "79e8241: upgrade note says development builds already carried 0.1.0, so format 4 and the type-rule/.gitignore scope decide; older or other-scope snapshots scan cold once, a format-4 same-scope snapshot can be served"
resolution: null
duplicate_of: null
---
CHANGELOG.md:218-219@d303dc1. engine_fingerprint mixes CARGO_PKG_VERSION, already 0.1.0 in development builds (crates/fdu-core/src/snapshot.rs:201-211), so a development build after the format-4 bump with the same scope writes a snapshot 0.1.0 serves.
