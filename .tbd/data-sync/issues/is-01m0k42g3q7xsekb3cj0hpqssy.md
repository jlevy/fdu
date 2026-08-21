---
type: is
id: is-01m0k42g3q7xsekb3cj0hpqssy
title: Name the layout rules and enforce them with a colour-vs-plain alignment test
kind: task
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0k41ks4s0nxzfxj3v141nx8
created_at: 2026-08-21T21:36:45.686Z
updated_at: 2026-08-21T21:57:19.370Z
closed_at: 2026-08-21T21:57:19.369Z
close_reason: Landed; make check green (24 suites, 129 goldens).
---
Write the layout rules down and make them hard to break, beside the colour role table:

- a column's width is measured on VISIBLE text; never pass a painted string to a width
  format specifier, because escape sequences count toward the width and silently eat the
  padding
- the shared column contract for a grouped row: size right-aligned in 10, share
  right-aligned in 6, label left-aligned to the section's widest label, then free-form
  detail
- one helper applies padding around `paint` so a renderer cannot get it wrong by writing
  `{:<N}` on painted text

Enforce it with a test that renders every grouped view twice, once with colour and once
without, strips the escapes from the coloured run, and asserts the two are identical. That
is the check the golden suite structurally cannot make, because goldens run under
NO_COLOR=1 and only ever see the uncoloured form.
