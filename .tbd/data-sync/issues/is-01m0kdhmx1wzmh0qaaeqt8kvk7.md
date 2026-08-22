---
type: is
id: is-01m0kdhmx1wzmh0qaaeqt8kvk7
title: Python API cannot render text; expose Report.render over the existing renderer
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-22T00:22:19.296Z
updated_at: 2026-08-22T00:22:19.296Z
---
The Python API cannot render text. There is no `render` in the package or the native stub:
the typed surface returns structured values and `as_dict()`, and stops.

A Python caller who wants fdu's own output must shell out to the binary -- the same
admission the console script already makes, since `fdu:_main` calls `_native.main()`, so
the `fdu` command the wheel installs has never exercised a line of the Python API.

Add `Report.render(format, color)` over the renderer the CLI already uses.

Found while planning the parity harness: without it, a shim serving the corpus's text
sessions would have to reimplement the bars, padding, headers, bound notes, footer, and
colour rules in Python -- hundreds of lines of duplicated presentation, and the harness
would then measure the reimplementation rather than the API.

Not test scaffolding. It is what lets Python be the CLI's equal rather than a data source
beside it.
