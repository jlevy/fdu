---
type: is
id: is-01m2et6m15tytpzabe7fsszz15
title: "PR #48 review FIX48-4: NotADirectory is state-dependent yet fails the whole read as InvalidArgumentError"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2et5s9yyv9e6tz783rp2m6c
created_at: 2026-09-14T01:58:40.932Z
updated_at: 2026-09-14T01:58:40.932Z
---
Low, from fa033c2. At f917cb7: read.rs:72-74, 425-427; opened_binding.rs:152-154 (maps to Python InvalidArgumentError). A Tree or RollUp naming a retained file fails the whole ReadRequest, including a Lookup that would have answered Present, and maps to InvalidArgumentError alongside static request-shape errors although it depends on index state at the pinned version. Design decision, not a defect. Disposition here: no behavior change; document NotADirectory as state-dependent on the Rust error and in the Python InvalidArgumentError docs. The per-projection-result versus whole-read-failure decision stays open on fdu-l89e. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5193206420
