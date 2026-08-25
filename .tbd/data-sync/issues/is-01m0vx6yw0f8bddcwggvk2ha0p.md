---
type: is
id: is-01m0vx6yw0f8bddcwggvk2ha0p
title: A walk budget, or the decision that fdu does not have one
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T07:30:01.728Z
updated_at: 2026-08-25T07:30:01.728Z
---
The interactive-client contract declares a file budget as *scope* -- it is validated as
positive, it is part of the scope fingerprint, and the reference provider truncates
discovery at it, reporting partial coverage with reason `budget` and a typed
resource-budget issue.

fdu has no walk budget. `CoverageReason::Budget` is declared and documented as unreachable,
and `ScanScope` carries depth, symlink, filesystem and hidden-admission facts but no entry
cap. So an fdu-backed provider given the same fingerprinted scope returns a different
inventory than the reference one: complete where the other is truncated. That is the same
defect shape as a fingerprinted-but-unimplemented scope axis, facing the other way.

Two ways out, and this bead is to pick one rather than let each side assume:

1. fdu grows a walk budget as scope -- fingerprinted like every other scope value, stopping
   discovery at the cap, marking coverage `Partial(Budget)`, and emitting a typed issue.
   The bound has to be a property of the walk rather than of a projection, because a budget
   that only truncates an answer leaves the tree read anyway and saves nothing.
2. The consumer drops the cap from its scope and its fingerprint, and bounds cost through
   the projection bounds it already mandates.

Option 1 makes `CoverageReason::Budget` reachable, which is the reason it was declared.
Option 2 is a product decision on the consumer's side, not fdu's.

Blocked on the joint answer; recorded so the adapter does not silently pick one.
