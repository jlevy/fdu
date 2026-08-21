---
type: is
id: is-01m0k42fs4g3x7bh8mkceb7ew2
title: The extensions view misaligns whenever colour is on
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0k41ks4s0nxzfxj3v141nx8
created_at: 2026-08-21T21:36:45.347Z
updated_at: 2026-08-21T21:57:19.070Z
closed_at: 2026-08-21T21:57:19.069Z
close_reason: Landed; make check green (24 suites, 129 goldens).
---
`render_text_types`, which draws the extensions view, pads the *painted* label:

    "{:>10}  {:<12} {} {}",  paint(&row.extension, STYLE_TYPE, color), ...

`paint` wraps the text in escape sequences, and `{:<12}` counts those toward the width, so
under colour the field is already "full" before any visible character is counted and the
padding collapses. Reported from a real terminal:

    673 MiB  (none) 6454 files
    218 MiB  .h  13239 files
    214 MiB  .js 15645 files
    95 MiB  .map 171 files

Colour off, the same view aligns, which is why the goldens (NO_COLOR=1) never caught it.

`render_text_metrics` already does it correctly: it measures `longest_label` on the
unpainted label and appends the padding *after* the painted text. That is the rule; this
renderer just does not follow it.
