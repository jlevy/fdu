---
type: is
id: is-01m0hg9fppyeggat22bf4m99me
title: Update help, SKILL, README, manifests, and goldens for the content axis
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01m0hfrm2xbqzdx4avgegcvf0t
created_at: 2026-08-21T06:31:48.693Z
updated_at: 2026-08-21T07:15:54.751Z
closed_at: 2026-08-21T07:15:54.750Z
close_reason: Implemented on claude/fdu-content-axis; make check green (24 suites, 114 goldens).
---
Update every surface that names the old vocabulary, in the same change as the code, per
Principle 12 (flags are part of benchmark identity):

- `--analyze` help text, which must state plainly that it opens and reads eligible files
- `AFTER_HELP` five-common-reports block
- SKILL.md
- README: the Five Common Reports table and the three-cost-layers section
- benchmark job manifests referencing `--analyze basic|documents`
- `crates/fdu/examples/perf_probe.rs` profile constructors
- the design doc: Principle 13 and the six-axis model

Re-record the golden sessions last, after the behavior beads land, and review the diff
rather than accepting it: an unexplained change in a block nobody meant to touch is a
finding, not noise.
