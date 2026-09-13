---
type: is
id: is-01m2ebdpn17ywtkyt2yk18wsxp
title: "PR #48 review PY-3: a negative id in a hand-built value raises OverflowError, not InvalidArgumentError"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:24.351Z
updated_at: 2026-09-13T21:40:24.351Z
---
Low. crates/fdu-py/src/opened_binding.rs:166-207. A negative int in EngineVersion, Continuation, or ScopeIdentity raises OverflowError during extraction, which _opened_call does not map to InvalidArgumentError. Fix: map OverflowError, or validate in __post_init__. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
