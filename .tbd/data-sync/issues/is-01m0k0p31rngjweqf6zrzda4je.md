---
type: is
id: is-01m0k0p31rngjweqf6zrzda4je
title: "Guard the guide against drift: assert it only names things that exist"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0k0nb25qmg8fpvaqybdmpc2
created_at: 2026-08-21T20:37:33.367Z
updated_at: 2026-08-21T20:49:07.658Z
closed_at: 2026-08-21T20:49:07.657Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 115 goldens).
---
The guide names flags, views, analyzers, and schema strings. Every one of those can go
stale silently -- and did: the schema bump to fdu.report/3 left "metric summaries use
fdu.report/2" in the help text, where it contradicted what the binary emits, and it
survived a full vocabulary sweep and `make check`.

tbd states the rule as "the menu must only name selectors that exist". Make it a test
rather than an intention:

- every `--flag` token the guide mentions resolves against the clap command
- every view and analyzer name it lists is accepted by the parsers
- no surface (guide, help, SKILL.md) names a schema string the binary does not emit

The `contract()` function in fdu-py is the existing precedent: it publishes each axis's
vocabulary and the Python tests assert parity against it.
