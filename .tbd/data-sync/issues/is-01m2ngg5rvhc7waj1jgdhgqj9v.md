---
type: is
id: is-01m2ngg5rvhc7waj1jgdhgqj9v
title: "PR #64 review RN64-9: notes omit symlinks never followed, hard links counted per path, allocated sizes by default"
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
created_at: 2026-09-16T16:23:49.274Z
updated_at: 2026-09-16T16:34:20.117Z
started_at: 2026-09-16T16:25:05.251Z
closed_at: 2026-09-16T16:34:20.116Z
close_reason: "ea30d7a: allocated bytes by default, symlinks listed but never followed and adding nothing to totals, hard links counted once per path (fdu-579b open); verified in source and on a scratch tree"
resolution: null
duplicate_of: null
---
docs/project/release-notes/0.1.0.md:111-148@d303dc1 (optional). ScanConfig::follow_symlinks false and true refused (crates/fdu-core/src/scan.rs:257,376); no inode deduplication (hardlink policy open as fdu-579b); --size defaults to allocated (crates/fdu/src/cli.rs:460).
