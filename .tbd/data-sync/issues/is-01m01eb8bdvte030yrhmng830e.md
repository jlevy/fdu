---
type: is
id: is-01m01eb8bdvte030yrhmng830e
title: Qualify partial macOS scans without masking permission errors
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - macos
  - validation
dependencies:
  - type: blocks
    target: is-01m01ed61j7yty2bqp0zw8v0xc
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:49:58.636Z
updated_at: 2026-08-15T00:51:01.809Z
---
Treat natural TCC-protected trees as a required product diagnostic without weakening fdu's strict partial-result semantics. Build a reproducible permission fixture where supported and retain the live Application Support reproduction as confidential local evidence. Compare fdu, fdu --allow-partial, and dust --print-errors for the set of unreadable directories, totals over readable entries, exit behavior, warning-rendering cost, and worker-policy behavior.

Acceptance: fdu continues to expose every skipped directory and to distinguish partial success from complete success; no speed claim uses an error-bearing or mutable sample; the diagnostic demonstrates whether permission/fallback paths influence scheduling; repeated warnings are measured and summarized without hiding detail from machine output; any UX change coordinates with fdu-oqoy rather than silently adopting dust's warning-plus-zero-exit behavior.
