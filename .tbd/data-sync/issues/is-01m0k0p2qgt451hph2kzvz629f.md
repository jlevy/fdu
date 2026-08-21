---
type: is
id: is-01m0k0p2qgt451hph2kzvz629f
title: Add --docs and reduce help to the flag reference
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0k0nb25qmg8fpvaqybdmpc2
created_at: 2026-08-21T20:37:33.039Z
updated_at: 2026-08-21T20:49:07.315Z
closed_at: 2026-08-21T20:49:07.315Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 115 goldens).
---
Drop `before_help`, so help opens with the description and usage as clap intends. Replace
`after_help` with one line: "Run `fdu --docs` for more help and important usage examples."

Add `--docs`, beside `--skill`, printing the guide: the report ladder, the two axes and
how they relate, more compositions, the six axes, content analysis, output and automation,
and exit status. Exempt it from the PATH requirement, as `--skill` already is, and add it
to the usage block.

Measured: -h 118 -> 59 lines, --help 190 -> 133, --docs 78.
