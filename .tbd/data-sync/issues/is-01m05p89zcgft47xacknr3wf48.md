---
type: is
id: is-01m05p89zcgft47xacknr3wf48
title: Lint and typecheck the benchmarks harness
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-15-fdu-performance-record-and-report.md
labels:
  - performance
dependencies: []
parent_id: is-01m045mb3wndcgrz7gxyw202sy
created_at: 2026-08-16T16:25:08.321Z
updated_at: 2026-08-16T16:25:08.321Z
---
`make python-check` runs ruff and basedpyright against crates/fdu-py only
(PYTHON_LINT_PATHS). The benchmarks/realtree harness -- 8,000 lines that generate the
ledger, decide accept/reject, and validate every artifact -- is neither linted nor
typechecked, only unit-tested.

Found the predictable consequence while adding the collision check: summary.py
annotated a return as Optional[Mapping[...]] without importing Optional. It survives
only because `from __future__ import annotations` makes annotations lazy strings; any
tool that evaluates them (typing.get_type_hints, a runtime validator) would raise
NameError. Fixed in passing.

The harness is the instrument the performance discipline rests on, so it deserves at
least the checks the library binding gets. Extend PYTHON_LINT_PATHS (or add a
benchmarks-scoped target) to cover benchmarks/, then fix what it surfaces. Expect the
pass to be noisy the first time; that is the argument for doing it rather than against.
