---
type: is
id: is-01m0xys6xmes768brpzn3gc7zt
title: "PR #48 review R15: define continuation lifetime in the joint contract"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex@spud10.local
labels: []
dependencies: []
parent_id: is-01m0xyqrr2t9q75j8v9q7v6kwj
hold: null
hold_until: null
created_at: 2026-08-26T02:35:57.235Z
updated_at: 2026-08-26T03:08:38.402Z
started_at: 2026-08-26T02:36:25.039Z
closed_at: 2026-08-26T03:08:38.402Z
close_reason: "Addressed in c4716ec; full disposition posted on PR #48; local make check and all 19 GitHub checks passed."
resolution: null
duplicate_of: null
---
PR #48 R15. Plan lines 494-510 and 664-691 do not state that handle-local continuation IDs die on root replacement or host-generation change.
