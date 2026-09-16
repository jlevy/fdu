---
type: is
id: is-01m0k4rpp91g28w9dwphgxj4mp
title: Rename --view all to --view full and define its membership
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels: []
dependencies: []
parent_id: is-01m0k4qrz1rb300efa1s5z86w6
created_at: 2026-08-21T21:48:53.320Z
updated_at: 2026-09-16T16:50:15.576Z
closed_at: 2026-09-16T16:50:15.574Z
close_reason: "a6b670c (PR #39): --view full is every summary view including largest and recent, excluding files (ViewSpec::full_report), --view all is rejected, and a documents view skipped without analysis is named. PR #62 carried full through the shipped text."
resolution: null
duplicate_of: null
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
