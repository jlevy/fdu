---
type: is
id: is-01m10nr925xb4ybt40q2pw7zpn
title: Revise MetaBrowser lifecycle, query, paging, count, and work values
kind: task
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nr9f2mkwdtp8ad88ms621
parent_id: is-01m0y1sfw7kwjprd6sfky281fj
created_at: 2026-08-27T03:55:52.771Z
updated_at: 2026-08-27T06:27:14.067Z
---
Align LifecyclePhase, CoverageReason, Coverage, Freshness, SourceKind, IndexState, WorkCounters, ReadRequest, query/result records, path issues, continuations, and exact-or-capped counts with the fdu architecture. Remove remaining_rows control flow, specify exact tree/flat order and one active change iterator, and keep all request and work bounds explicit.
