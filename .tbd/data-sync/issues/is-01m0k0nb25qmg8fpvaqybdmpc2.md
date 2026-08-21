---
type: is
id: is-01m0k0nb25qmg8fpvaqybdmpc2
title: "Discovery surface: a --docs guide, and help that is a flag reference again"
kind: epic
status: closed
priority: 2
version: 6
labels: []
dependencies: []
child_order_hints:
  - is-01m0k0p2qgt451hph2kzvz629f
  - is-01m0k0p31rngjweqf6zrzda4je
  - is-01m0k0p3c5mvr26tvmssqx6xgx
  - is-01m0k0p3pp4zgv78ymf9wc71t1
created_at: 2026-08-21T20:37:08.804Z
updated_at: 2026-08-21T20:49:08.654Z
closed_at: 2026-08-21T20:49:08.653Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 115 goldens).
---
`--help` opened with a page of prose above the tool's own description, because the report
examples lived in clap's `before_help`. A reader met the examples before learning what the
command was, and `-h` -- clap's concise form -- carried the whole thing at 118 lines.

Split the surface: `--help` is the flag reference, and one guide holds what the
before/after blocks used to say, reached by a single `--docs` flag. No subcommands.

The guide is written for both readers and agents: complete, self-contained, and every
example copy-pasteable. tbd's docs surface is the model for the discipline, not the shape
-- "the wording lives in one place and the two surfaces cannot drift", and "the menu must
only name selectors that exist".
