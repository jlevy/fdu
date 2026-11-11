---
type: is
id: is-01m75hvvn1xeqc0jzjhmhgvcyq
title: "PR #209 H6: a nested filesystem mounted under a new number reads 'the filesystem was no longer mounted here'"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m75hvp13zk79cex02q85ffc1
hold: null
hold_until: null
created_at: 2026-11-11T14:58:18.913Z
updated_at: 2026-11-11T16:11:15.507Z
started_at: 2026-11-11T14:58:37.253Z
closed_at: 2026-11-11T16:11:15.507Z
close_reason: "Fixed in 2886568e: one rule for root and nested rows, reason 'not found on the device the walk recorded (unmounted, expired, or renumbered since)'; usage guide note in 9d74ace1."
resolution: null
duplicate_of: null
---
Low. query/query_volumes.rs:128-131, :338-344. Fix: one rule for root and nested rows: a device found now that is not the recorded one reads capacity unknown with an accurate reason. Review: https://github.com/jlevy/fdu/pull/209#issuecomment-6792308952 (PR #209).
