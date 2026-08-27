---
type: is
id: is-01m10nr925xb4ybt40q2pw7zpn
title: Revise MetaBrowser lifecycle, query, paging, count, and work values
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nr9f2mkwdtp8ad88ms621
parent_id: is-01m0y1sfw7kwjprd6sfky281fj
created_at: 2026-08-27T03:55:52.771Z
updated_at: 2026-08-27T07:56:15.559Z
closed_at: 2026-08-27T07:56:15.558Z
close_reason: "Completed in MetaBrowser commit 45266a8: the measured revised contract is frozen in architecture, code, and the closed conformance registry; native fdu implementation can now target it."
resolution: null
duplicate_of: null
---
Align LifecyclePhase, CoverageReason, Coverage, Freshness, SourceKind, IndexState, WorkCounters, ReadRequest, query/result records, path issues, continuations, and exact-or-capped counts with the fdu architecture. Remove remaining_rows control flow, specify exact tree/flat order and one active change iterator, and keep all request and work bounds explicit.
