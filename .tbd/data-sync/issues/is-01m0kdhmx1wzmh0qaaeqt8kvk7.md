---
type: is
id: is-01m0kdhmx1wzmh0qaaeqt8kvk7
title: Python API cannot render text; expose Report.render over the existing renderer
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/done/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-22T00:22:19.296Z
updated_at: 2026-08-23T00:05:56.691Z
closed_at: 2026-08-22T06:13:23.587Z
close_reason: "Report.render landed in PR #40. Index.report binds a renderer onto the frozen Report; a detached Report refuses to render rather than diverging from the CLI. Format added to contract() so the StrEnum parity assertion covers it. Verified across 3 views x 4 formats against the built wheel and the sdist."
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

## Notes

Decided: report.render(Format, color: bool) -> str, a method beside as_dict(). Format joins the exported StrEnums; color is a plain bool because resolving auto means asking about stdout, which the caller owns. Body only -- the performance footer stays CLI-transient, so every text session deviates by that one line, recorded as a single class with a count rather than ~94 identical hunks.
