---
type: is
id: is-01m0k2dnpdc8h6kbpqj2ppmt2c
title: "A named text styling system: ALL CAPS headings, no colons, one colour role table"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-21T21:07:54.700Z
updated_at: 2026-08-21T21:16:04.327Z
closed_at: 2026-08-21T21:16:04.326Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 129 goldens).
---
Name the terminal styling rules in one place instead of leaving them as five loose
constants and a convention nobody wrote down.

The system:
- headings are ALL CAPS, no trailing colon, in the heading colour
- colour appears only when the destination is a live terminal, and never in machine
  formats or under NO_COLOR
- every other role (warning, error, cause, telemetry, rule) keeps its existing colour but
  is named as a role rather than a colour

Two things to fix against it:
- clap appends a colon to every `help_heading`, so help reads "SCOPE:" while the report
  reads "SUMMARY". The rendering path is already ours -- `error.render()` into
  `write_styled`, which walks the output line by line -- so the colon can be dropped there.
- clap's built-in "Arguments" and "Options" headings are Title case; give those args an
  explicit heading so no default leaks through.

Also drop the wrap cap from 160 to 120 columns.
