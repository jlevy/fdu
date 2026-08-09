---
type: is
id: is-01kzm9qxbh857h2rym0q39c4te
title: Bind public watch application to its indexed root
kind: bug
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - pr-review
  - concurrency
dependencies:
  - type: blocks
    target: is-01kzm3t12dcq5h7n92xztnhcyd
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T22:19:22.864Z
updated_at: 2026-08-09T22:22:39.085Z
closed_at: 2026-08-09T22:22:39.084Z
close_reason: Removed the unrooted public application capability. The supported Watcher::apply_next path checks watcher/index root identity before consuming an intent; the internal generic-driver seam is test-only, next_observation documents its unrooted nature, and the mismatch-without-consumption regression passes in the 145-test all-feature library suite.
---
PR review found that the free public apply_observation helper accepts an unrooted Observation and an arbitrary IndexHandle, unlike Watcher::apply_next. Remove or redesign that unsupported public boundary so watch delivery cannot silently target a different tree; retain an explicit root mismatch test on the supported driver.
