---
type: is
id: is-01m0hg9ed5z364cssftdx592jw
title: Derive the default view from the analyzer set
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01m0hfrm2xbqzdx4avgegcvf0t
created_at: 2026-08-21T06:31:47.365Z
updated_at: 2026-08-21T07:15:53.786Z
closed_at: 2026-08-21T07:15:53.785Z
close_reason: Implemented on claude/fdu-content-axis; make check green (24 suites, 114 goldens).
---
Derive the default view from the requested analyzer set; an explicit `--view` always
wins. A view may never enable an analyzer (that would let a display choice authorize
filesystem reads); the reverse is free, because it re-projects state already paid for.

  empty            -> tree        (unchanged; the du-replacement answer)
  lines only       -> families    (broadest grouping that shows line counts)
  code, no words   -> languages   (the view whose rows carry SLOC)
  words, no code   -> documents   (the view whose rows carry text volume)
  code and words   -> families    (the one view showing both metric groups)

Fixes the defect this epic exists for: today `fdu --analyze all PATH` reads every
eligible file and prints a report byte-identical to `fdu PATH`, differing only in the
performance footer.

Changes output for every existing `--analyze` invocation that did not pass `--view`, so
the goldens are re-recorded and the diff reviewed as the record of the change.
