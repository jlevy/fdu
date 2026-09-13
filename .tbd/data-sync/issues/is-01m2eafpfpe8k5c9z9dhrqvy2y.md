---
type: is
id: is-01m2eafpfpe8k5c9z9dhrqvy2y
title: "Address review: PR #51 — commit-pipeline optimizations and the control-observation gate"
kind: task
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
child_order_hints:
  - is-01m2eag5hr11xx52vv30jzswaa
  - is-01m2eag61zh81wz4tcszntdk30
  - is-01m2eag6gyvzzt41f6xagdz2ap
  - is-01m2eag6x0a7xr583z56k4ve7v
created_at: 2026-09-13T21:24:01.127Z
updated_at: 2026-09-13T21:24:17.951Z
---
Formal review 5192254822 on PR #51 (https://github.com/jlevy/fdu/pull/51#pullrequestreview-5192254822) at head 19c0d73, plus PR #50 review 5192254897 (https://github.com/jlevy/fdu/pull/50#pullrequestreview-5192254897), whose single finding is the same PLAN-3 and whose text now lives on #51's branch.

Findings and their beads:
- COMMIT-1 (High) directional snapshot acceptance collides with reconcile's strict scope check -> existing fdu-vev7 (fixed on #52 as bcb27ca; ported down)
- COMMIT-2 (High) watch event path ignores read_controls -> new child bead
- COMMIT-3 (High) Python one-shot calls still observe controls; policy lives in the CLI -> existing fdu-etfj
- COMMIT-4 (Low) prepared paths no longer byte-canonical -> existing fdu-lksd (fixed on #52 as 3bdfbb2; ported down)
- PLAN-1, PLAN-2, PLAN-3 (Medium) plan text -> new child beads

Deferred: opened roots always observe controls and can still abort on control volume (4 MiB table bound and 16 KiB per-line bound) -> fdu-1onj.
