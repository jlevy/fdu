---
type: is
id: is-01m75hvtsaxd65a6vfqq1t5cpy
title: "PR #209 H5: a reconciliation records the directory above its subtree from the stat alone, undoing the walk's record"
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
created_at: 2026-11-11T14:58:18.026Z
updated_at: 2026-11-11T16:11:16.307Z
started_at: 2026-11-11T14:58:38.209Z
closed_at: 2026-11-11T16:11:16.307Z
close_reason: "Fixed in 2886568e: push_directory_attributes and the watch's directory restate go through entered_directory_attrs with the held parent's device; targeted_refresh_probe.py: data/trig one row of 3 files, 0 requests."
resolution: null
duplicate_of: null
---
Low. scan.rs:7411-7433 (push_directory_attributes); watch.rs:1076-1100 has the same shape. Fix: pass through entered_directory_attrs with the held parent's device. Review: https://github.com/jlevy/fdu/pull/209#issuecomment-6792308952 (PR #209).
