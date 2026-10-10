---
type: is
id: is-01m4kwner1kf0cqmqyy2q1nrtc
title: "report_roots: confirm an alias match from an unverified index, as prepare_roots_report does"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
created_at: 2026-10-10T21:49:20.000Z
updated_at: 2026-10-10T21:49:20.000Z
---
Review E3 on PR #192 (jlevy/fdu#192). report_roots refuses with RootReachedInside on directory identities read from an index loaded without verification (Delivery::stale_ok, Index::mark_unverified: freshness Stale, source Cached). prepare_roots_report confirms such a match with one symlink_metadata (reached_root, execution.rs) and answers when the inode was reused or the device renumbered since the snapshot. Two routes then disagree for one request and history (Caching Improves Performance, Never Semantics). Rare: needs inode reuse or a remount. #192 documents the limitation under report_roots' Errors. Fix: for an unverified index, confirm a match with reached_root's one read and say on report_roots that this is its only read. Test: report_roots_confirms_a_match_from_a_cache_only_index (Unix): stamp a snapshot, replace the matching directory, expect acceptance.
