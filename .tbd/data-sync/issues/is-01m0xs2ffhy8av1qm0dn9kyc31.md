---
type: is
id: is-01m0xs2ffhy8av1qm0dn9kyc31
title: Implement the opened-root inventory engine rewrite
kind: epic
status: in_progress
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/48
    at: 2026-08-26T01:54:36.597Z
delegate: codex@spud10.local
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
child_order_hints:
  - is-01m0xyqrr2t9q75j8v9q7v6kwj
hold: null
hold_until: null
created_at: 2026-08-26T00:56:09.456Z
updated_at: 2026-08-26T03:08:38.012Z
started_at: 2026-08-26T01:54:35.977Z
---
Implement the clean opened-root and streaming design from the linked plan in merge-sized slices from current main. Preserve the one-shot engine and CLI defaults; first establish exact commits and ownership, then the minimal opened-root lifecycle, then adopt it through MetaBrowser's provider conformance boundary.

## Notes

Implementation is proceeding on the single long-lived draft branch codex/opened-root-inventory-rewrite in PR #48. Initial commit 51d9b47 contains the complete PR #47 review and clean cross-repository plan. Review-response commit c4716ec addresses every R1-R15 and S1-S2 finding, adds four Phase 1 checkpoints and the Phase 3A unchanged-contract measurement gate, and passed the full local handoff gate plus all 19 GitHub checks. Keep the PR draft through Phase 4; land independently green phase checkpoints on this branch and coordinate MetaBrowser PR #74 against exact counterpart revisions.
