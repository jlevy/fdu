---
type: is
id: is-01m2nm3bm0v90ypqfhxeeet6r3
title: "PR #69 review 69-1: TODO.md counts fdu-yov0 as six open beads and two finished specs"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2nm36wpq4crkj85j40kmnn0
created_at: 2026-09-16T17:26:43.584Z
updated_at: 2026-09-16T17:37:29.419Z
closed_at: 2026-09-16T17:37:29.418Z
close_reason: "d3feb73 (PR #69): TODO.md counts fdu-yov0 from its beads (1 open, fdu-c2ml), names fdu-747k for the move, and counts three shipped specs in active/; true before and after PR #71."
resolution: null
duplicate_of: null
---
TODO.md:26, :53, :60-61@fe5cb70 (PR #69). The fdu-yov0 row says six open beads, the plan row says "beads open", and :60 counts two finished specs in active/. Six of fdu-yov0's seven children are closed (qbwf, xc1v, j1dc, c1qh, 5akc on 2026-09-16 citing a6b670c / PR #39; k4ad under PR #62); only fdu-c2ml is open. Three specs in active/ have shipped, and the move is fdu-747k. Word it from the beads so it holds whether or not PR #71 has merged.
