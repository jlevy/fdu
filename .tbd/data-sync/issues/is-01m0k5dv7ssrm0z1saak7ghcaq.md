---
type: is
id: is-01m0k5dv7ssrm0z1saak7ghcaq
title: Every view renders in every format is untested for half the matrix
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels: []
dependencies: []
parent_id: is-01m0k4qrz1rb300efa1s5z86w6
created_at: 2026-08-21T22:00:26.104Z
updated_at: 2026-08-21T22:34:57.550Z
---
Principle 9 says formats are serializations, not features, and that every view renders in
every format. Audited 2026-08-21, that is a claim rather than a test:

           text  json  jsonl  yaml
  tree      x     x     x      -
  types     x     x     x      -
  extensions x    -     -      -
  families  x     x     -      -
  languages x     x     x      -
  documents x     x     x      x
  files     x     -     -      -
  summary   x     x     -      x

`extensions` and `files` are exercised in text alone, and most of the yaml column is
empty. A view that renders in three formats and panics in the fourth would ship.

Table-driven unit test rather than goldens: crossing every view with every format as
golden sessions would add dozens of blocks that mostly restate one another, while a test
asserts the property directly. Distinct from fdu-c2ml, which asks whether what is rendered
is *valid*; this asks whether it renders at all.

## Notes

CORRECTION: the matrix WAS already tested by report_format::every_view_renders_in_every_format over 8 views x 4 formats. The audit that filed this measured GOLDEN coverage and reported it as test coverage -- a real distinction, but not the one claimed. Genuine gap, now closed: the test used a hand-written view list that would not have included largest/recent, and asserted only non-emptiness. It is now driven from ALL_TEST_VIEWS and asserts a schema is present.
