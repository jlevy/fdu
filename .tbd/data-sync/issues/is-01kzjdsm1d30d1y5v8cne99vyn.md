---
type: is
id: is-01kzjdsm1d30d1y5v8cne99vyn
title: Restrict persisted snapshot permissions
kind: bug
status: closed
priority: 1
version: 4
labels:
  - security
  - snapshot
dependencies: []
parent_id: is-01kzjceqsx74x63350s7hjb8q0
created_at: 2026-08-09T04:51:44.300Z
updated_at: 2026-08-09T05:10:37.454Z
closed_at: 2026-08-09T05:10:37.454Z
close_reason: "Implemented in commit 5014b13 with focused regression tests; local make check and all PR #1 checks pass on Linux, macOS, Windows, MSRV 1.85, dependency policy, docs, and Python wheel smoke."
---
Snapshot files contain a full filesystem inventory. Create replacement files with owner-only permissions on Unix and prove the installed snapshot does not inherit a world-readable umask default.

## Notes

Red test reproduced mode 0644 under the normal umask. Exclusive sibling temp files now use mode 0600 on Unix; rename installs the protected inode, with a regression test on the final snapshot.
