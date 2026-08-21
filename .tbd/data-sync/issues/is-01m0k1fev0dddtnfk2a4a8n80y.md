---
type: is
id: is-01m0k1fev0dddtnfk2a4a8n80y
title: "Golden sessions for the content axis: derived views, --view all, both notes, the totals errors"
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m0k1hjk2w50cmaxrc3rwmvc8
created_at: 2026-08-21T20:51:24.639Z
updated_at: 2026-08-21T21:05:36.182Z
closed_at: 2026-08-21T21:05:36.180Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 129 goldens).
---
Audit result: the content-axis behavior is covered by Rust unit tests but not by a single
golden session. Every golden command passes `--view` explicitly, so none of these have a
text contract:

- the default view derived from the analyzer set -- the headline fix, and the one whose
  whole point is what a user sees without naming a view
- `--view all` and the note naming what it skipped
- the note when a selected view displays none of the requested analysis
- set composition (`--analyze code,words`) and order-independence
- the new usage errors: `none,code`, `all,code`, `--view all,tree`, and the retired
  `basic`/`documents`/`full` spellings

Golden testing guidelines: session tests reveal broad state and confirm expected results
while making unexpected changes obvious; unit tests verify narrow expectations. The
user-visible shape of a report is exactly the former. The runbook adds the sizing rule --
concise, realistic, end-to-end, smallest fixture exposing the largest useful surface -- so
this is a couple of sessions, not one per case.
