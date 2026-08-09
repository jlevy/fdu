---
type: is
id: is-01kzjcpgpp0h1c3hqpr8y22yca
title: Reject snapshot payload corruption with an integrity checksum
kind: bug
status: closed
priority: 1
version: 3
labels:
  - persistence
  - correctness
dependencies: []
parent_id: is-01kzjceqsx74x63350s7hjb8q0
created_at: 2026-08-09T04:32:34.005Z
updated_at: 2026-08-09T05:10:37.403Z
closed_at: 2026-08-09T05:10:37.402Z
close_reason: "Implemented in commit 5014b13 with focused regression tests; local make check and all PR #1 checks pass on Linux, macOS, Windows, MSRV 1.85, dependency policy, docs, and Python wheel smoke."
---
The bootstrap snapshot validates structure and a trailing marker but has no payload integrity check. A plausible bit flip in an attribute or path byte can still parse and silently change totals. Add a bounded pre-parse checksum, bump the disposable pre-release format version, preserve structural-validation coverage, and test plausible corruption.
