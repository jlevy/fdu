---
type: is
id: is-01m2eafpfpe8k5c9z9dhrqvy2y
title: "Address review: PR #51 — commit-pipeline optimizations and the control-observation gate"
kind: task
status: closed
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
child_order_hints:
  - is-01m2eag5hr11xx52vv30jzswaa
  - is-01m2eag61zh81wz4tcszntdk30
  - is-01m2eag6gyvzzt41f6xagdz2ap
  - is-01m2eag6x0a7xr583z56k4ve7v
  - is-01m2esgpfc4tzvj2yfandq5cvw
  - is-01m2esgq6gsyxak08nat4vxykw
created_at: 2026-09-13T21:24:01.127Z
updated_at: 2026-09-14T01:46:43.279Z
closed_at: 2026-09-13T22:08:34.792Z
close_reason: "All seven findings fixed on PR #51 (c0729ce, 50e6ca5, 6d2d964, 046c9ec, a69b95e, b36d5aa, 51154f9); CI 19/19 green at 51154f9; dispositions posted on #51 and #50. COMMIT-1/COMMIT-4 tracked on fdu-vev7/fdu-lksd, COMMIT-3 on fdu-etfj (closes at merge). Opened-root/open/--watch abort on control volume deferred to fdu-1onj."
resolution: null
duplicate_of: null
---
Formal review 5192254822 on PR #51 (https://github.com/jlevy/fdu/pull/51#pullrequestreview-5192254822) at head 19c0d73, plus PR #50 review 5192254897 (https://github.com/jlevy/fdu/pull/50#pullrequestreview-5192254897), whose single finding is the same PLAN-3 and whose text now lives on #51's branch.

Findings and their beads:
- COMMIT-1 (High) directional snapshot acceptance collides with reconcile's strict scope check -> existing fdu-vev7 (fixed on #52 as bcb27ca; ported down)
- COMMIT-2 (High) watch event path ignores read_controls -> new child bead
- COMMIT-3 (High) Python one-shot calls still observe controls; policy lives in the CLI -> existing fdu-etfj
- COMMIT-4 (Low) prepared paths no longer byte-canonical -> existing fdu-lksd (fixed on #52 as 3bdfbb2; ported down)
- PLAN-1, PLAN-2, PLAN-3 (Medium) plan text -> new child beads

Deferred: opened roots always observe controls and can still abort on control volume (4 MiB table bound and 16 KiB per-line bound) -> fdu-1onj.
