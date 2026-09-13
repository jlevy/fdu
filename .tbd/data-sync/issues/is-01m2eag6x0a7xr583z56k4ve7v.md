---
type: is
id: is-01m2eag6x0a7xr583z56k4ve7v
title: "PR #51/#50 review PLAN-3: the plan names the CLI, not the shared planner, as where control observation is decided"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2eafpfpe8k5c9z9dhrqvy2y
created_at: 2026-09-13T21:24:17.951Z
updated_at: 2026-09-13T21:24:17.951Z
---
PR #51 review PLAN-3 and PR #50 review PLAN-3 (Medium), https://github.com/jlevy/fdu/pull/50#pullrequestreview-5192254897. The fdu-etfj implementation row (plan line 2185 at 19c0d73) names crates/fdu/src/cli.rs (and at #50's head, crates/fdu/Cargo.toml and scan.rs) as where the policy is decided, never execution::plan_report / prepare_report, the one-shot planner the CLI and Python both call.

Fix on #51's branch (the text now lives there) in step with COMMIT-3: name the shared planner, state the acceptance per surface.
