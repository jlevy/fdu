---
type: is
id: is-01m0k4rpp91g28w9dwphgxj4mp
title: Rename --view all to --view full and define its membership
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels: []
dependencies: []
parent_id: is-01m0k4qrz1rb300efa1s5z86w6
created_at: 2026-08-21T21:48:53.320Z
updated_at: 2026-08-21T21:50:33.794Z
---
`--view all` becomes `--view full`: every summary view, in table order, now including both
`largest` and `recent`, and excluding `files`.

The rename is not cosmetic. `--analyze all` means literally every analyzer; a view total
cannot mean literally every view once one view is an unbounded enumeration. `full` reads
as "the full report" rather than "every value", and the different word marks the different
semantics -- which is the distinction that was wanted when the two totals were first
named.

Keep the omission note: `documents` is still skipped without content analysis, and a
digest that drops a section must say so.
