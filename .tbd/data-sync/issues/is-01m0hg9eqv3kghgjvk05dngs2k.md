---
type: is
id: is-01m0hg9eqv3kghgjvk05dngs2k
title: Add --view all with profile-aware expansion and an omission note
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01m0hfrm2xbqzdx4avgegcvf0t
created_at: 2026-08-21T06:31:47.705Z
updated_at: 2026-08-21T07:15:54.028Z
closed_at: 2026-08-21T07:15:54.027Z
close_reason: Implemented on claude/fdu-content-axis; make check green (24 suites, 114 goldens).
---
Add `all` to `--view`, expanding to every view the requested analyzer set can answer, in
the canonical table order. `all` is a total and must appear alone.

`documents` requires analysis, so `--view all` without it would otherwise fail the whole
run over one unsatisfiable view. Instead render what is satisfiable and name what was
skipped:

  note: omitted documents - requires --analyze words or all

Machine formats need no new field: the `reports` array already enumerates exactly which
views were produced, so a consumer reads the omission from what is absent. The note is a
human-format affordance only. Add a test pinning that `--view all` does not change either
report schema.
