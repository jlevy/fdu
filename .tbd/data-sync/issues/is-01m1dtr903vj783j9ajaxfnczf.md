---
type: is
id: is-01m1dtr903vj783j9ajaxfnczf
title: Prove one-shot parity and add deterministic regression guards
kind: task
status: in_progress
priority: 0
version: 16
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - validation
dependencies:
  - type: blocks
    target: is-01m1x444q4jz0680n8a057r5z8
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
child_order_hints:
  - is-01m1edc4xady6k86e0hsbzfsk1
  - is-01m1eek06tcb89yygyc1xz2yz5
  - is-01m1egf3aa4wt4kc2z5qmhspqp
  - is-01m1egxbrdj757jr3bk8bhv1ce
  - is-01m1ejqfv4khft8mbkfw7f3q0f
  - is-01m1ekg6ewkj2mr9wf1xs9g01y
hold: null
hold_until: null
created_at: 2026-09-01T06:33:23.201Z
updated_at: 2026-09-07T06:15:09.367Z
started_at: 2026-09-01T11:13:09.191Z
---
Re-profile after every accepted experiment, close only profile-named residual costs, meet the plan wall/component/allocation thresholds on control-free and control-rich real trees, add negative-tested per-entry allocation and detached zero-work guards, run the full and cross-platform gates, and record every experiment.

## Notes

2026-09-06 final review: review defects fixed and stack refreshed through 95cd4b0; all 19 checks passed on every restacked descendant. Cross-revision provenance fix 0bfb2cf passed local make check (224 realtree tests), CI pending with one Windows action-fetch failure being retried. Exact historical timing control b75bf85 is preserved. Measurement-only controls: historical allocation boundaries 2010fa3 (5 probe tests), c638 structural baseline plus corrected opened oracle 115ff4f (11 tests). No new timing claims yet. Fresh eligibility audit: live source tree 97,587 entries qualifies; Rust minimal toolchain 298 entries and pnpm content store 24,339 are too small; SDK/CommandLineTools exceed density limit; uv archive 176,105 is dense but includes 14 control files. Checking a naturally control-free dependency subtree next. No criterion relaxed; source checkouts share one Cargo target. Prior exp099/101 remain exploratory rather than final proof.
