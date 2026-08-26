---
type: is
id: is-01m0y1sfedr9qf3sc7e4bf6fd7
title: Measure a disposable fdu adapter against the unchanged MetaBrowser contract
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sfw7kwjprd6sfky281fj
  - type: blocks
    target: is-01m0y1shykye8sc7h7e9rkk6kh
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:31.692Z
updated_at: 2026-08-26T03:28:58.485Z
---
On MetaBrowser PR #74, build a deliberately disposable adapter against the unchanged provider protocol. Instrument row materialization, sorting, scans, totals, latency, memory, and route-visible ordering on the representative corpus; publish evidence, retain the harness, and delete naive replica and aggregation code.
