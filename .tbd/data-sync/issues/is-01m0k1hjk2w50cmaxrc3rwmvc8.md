---
type: is
id: is-01m0k1hjk2w50cmaxrc3rwmvc8
title: "Terminal output cleanups: one styling system, readable help, and the golden gap"
kind: epic
status: closed
priority: 2
version: 8
labels: []
dependencies: []
child_order_hints:
  - is-01m0k1j6csq8hat2qeq8syaywh
  - is-01m0k1j7136m6t1cxhkreaz2tm
  - is-01m0k1fev0dddtnfk2a4a8n80y
  - is-01m0k1n8pr9wz583w5g3dh4bak
  - is-01m0k1n932y3wkvzawac0d4tjr
created_at: 2026-08-21T20:52:34.017Z
updated_at: 2026-08-21T21:05:37.762Z
closed_at: 2026-08-21T21:05:37.761Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 129 goldens).
---
Two readability problems in `--help`, and one gap in what the golden suite pins about the
content axis. Grouped because the first two both change help output and therefore land in
the same golden re-record.
