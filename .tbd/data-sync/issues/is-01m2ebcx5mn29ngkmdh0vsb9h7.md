---
type: is
id: is-01m2ebcx5mn29ngkmdh0vsb9h7
title: "PR #48 review PY-2: the session goldens' final record is a literal, not an observation"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:58.259Z
updated_at: 2026-09-13T23:38:41.677Z
closed_at: 2026-09-13T23:38:41.677Z
close_reason: "Fixed in e8a9707: final records derive from a test-only observation of each root (joined, workers, waiters, continuations, stored close outcome); all five goldens changed only in final records; verified locally and in CI."
resolution: null
duplicate_of: null
---
Medium. crates/fdu-core/src/opened/golden_tests.rs:135, 193, 354, 406, 497 all record 'joined=true workers=0 waiters=0 continuations=0' as a constant; the plan (:2196-2200) defines final as observed state. Fix: a cfg(test) accessor on OpenedState for worker, waiter, and continuation counts, and derive final from it plus the actual close outcome; update goldens under the plan's golden update discipline. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
