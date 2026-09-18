---
type: is
id: is-01m2phzd7cxjn134d9t78xzw6d
title: "Decide: Rust API extensibility before 0.1.0 (#[non_exhaustive] or document 0.2 changes)"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - release
  - decision
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:51.435Z
updated_at: 2026-09-18T03:07:28.152Z
closed_at: 2026-09-18T03:07:28.152Z
close_reason: "Implemented on PR #87: SIGINT, registry READMEs, caret pins, version stamp/LF, 0.2 API note, 200ms interval, transient-summary --no-gitignore, python-smoke --python, JSON 2^53."
---
Exhaustive public types that planned work would extend, each a breaking change within 0.1.x:
opened-root `ReadProjection`, `ProjectionResult`, `ProjectionRefusal`, `LimitedProjection`, `IssueKind`,
`ImpactDomain`, `Error`, public-field structs `ReportRequest`, `TreePage`, `ReadResponse`; `RollUp`,
`Provenance`, `StateTransition`, `ReportSource`, `Attrs`, `Query`. Planned additions: fdu-fka6,
fdu-1mj4, FSEvents report source, checkpoint slice 2, and the analysis-request fix (fdu-gija).

Options: add `#[non_exhaustive]` now (touches fdu CLI and fdu-py match arms), or a CHANGELOG line that
these types change in 0.2 under the 0.x rule. User decision.
