---
type: is
id: is-01m0k1n932y3wkvzawac0d4tjr
title: Style the --docs guide with the shared system, live terminals only
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0k1hjk2w50cmaxrc3rwmvc8
created_at: 2026-08-21T20:54:35.361Z
updated_at: 2026-08-21T21:05:37.454Z
closed_at: 2026-08-21T21:05:37.453Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 129 goldens).
---
`--docs` writes plain text with no styling at all, so on a live terminal its section
headers do not match the report's. Give it the shared header style and the same
live-terminal gate, and leave it plain when redirected or when NO_COLOR is set.

Depends on the shared system being defined first.
