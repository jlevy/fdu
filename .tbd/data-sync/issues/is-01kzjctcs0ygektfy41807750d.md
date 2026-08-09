---
type: is
id: is-01kzjctcs0ygektfy41807750d
title: Reject reconciliation when semantic scope does not match the index
kind: bug
status: closed
priority: 1
version: 5
labels:
  - correctness
  - api
  - pr-review
dependencies: []
parent_id: is-01kzjceqsx74x63350s7hjb8q0
created_at: 2026-08-09T04:34:41.055Z
updated_at: 2026-08-09T05:10:37.441Z
closed_at: 2026-08-09T05:10:37.441Z
close_reason: "Implemented in commit 5014b13 with focused regression tests; local make check and all PR #1 checks pass on Linux, macOS, Windows, MSRV 1.85, dependency policy, docs, and Python wheel smoke."
---
PR thread discussion_r3742613558: mutation APIs accepted ScanConfig state inconsistent with Index::scope(), and restricted watch/reconcile paths could apply entries excluded by cold scan. Enforce exact scope matching before mutation or invalidation drain; reject the applying watch driver for max-depth/one-filesystem until event filtering exists; reject subtree requests beyond depth, below mount boundaries, or through symlink ancestors; make max_depth=0 consistently root-only; and reject apply_next before consuming an event.

## Notes

Implemented with regression tests across observation-only, direct, shared/pending, subtree, and watch entry points.
