---
type: is
id: is-01m0k3ta5nqsq1hdnnn3bqmydh
title: The files view truncates silently while the tree view marks it
kind: bug
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels: []
dependencies: []
created_at: 2026-08-21T21:32:17.460Z
updated_at: 2026-08-21T21:50:34.699Z
---
`--view files` stops at `--limit` (default 10) and prints nothing to say it truncated.
Reproduced on a 192,871-entry tree: 10 rows, no marker, no count -- the listing simply
ends and looks complete.

  $ fdu --view files .
  ...
  .agents/skills/tbd
  Performance: walked 180,199 files / 1.9 GiB; ...

The tree view already prints a `…` marker for entries it drops, so the two views disagree
about the same situation. This is the honesty rule the design doc states -- never present
a value as complete when it is not, and name an omission rather than hide it -- and it is
the same rule that made `--view all` report the view it skipped.

Fix: mark a truncated flat listing, and say how much was dropped, since a bare `…` on a
192k-entry tree tells you less than a count does.

Related: `--limit` is documented as "Entries to show per directory", which is accurate for
`tree` and wrong for `files`, where there is no per-directory grouping and the number caps
the whole listing. Worth a word in the help text.

Also worth deciding separately: `files` lists directories as well as files. That follows
the spec, which defines it as a flat listing of matching entries, and `--kind file`
narrows it -- but the name invites the other reading.
