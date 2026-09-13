---
type: is
id: is-01m2ebdqgp1sz7kzknpewg7r7x
title: "PR #48 review PY-5: the opened-root golden lint lacks the plan's duration check"
kind: task
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:25.237Z
updated_at: 2026-09-13T23:38:42.567Z
closed_at: 2026-09-13T23:38:42.566Z
close_reason: "Fixed in 30bd3ea: the opened-root golden lint rejects fractional durations and raw instants, with node tests. CI green."
resolution: null
duplicate_of: null
---
Low. scripts/check-opened-root-goldens.mjs:45-60. The plan (:2224-2226) says the lint rejects wall-clock durations; the script has no such check. Fix: add it, or amend the plan. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
