---
type: is
id: is-01m0k4ghby3abtwzpa59esx7xq
title: Document how to choose a default, and the truncation contract
kind: task
status: closed
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-08-21T21:44:25.724Z
updated_at: 2026-08-21T21:57:18.763Z
closed_at: 2026-08-21T21:57:18.761Z
close_reason: Landed; make check green (24 suites, 129 goldens).
---
The design doc says a great deal about correctness and honesty and nothing about how to
choose a default, which is how `--view files` shipped defaulting to name-ascending order
*and* ten rows: two conventional choices that do not compose, producing "the ten
alphabetically-first entries" of a 192,871-entry tree. That answers no question.

Neither choice was careless in isolation. Name order is what ls, find, and fd do, and ten
rows keeps a report readable. They survived review precisely because each looked normal.

Two principles to add:

1. A default must answer a question you can state in one sentence. Inherit a convention
   only with the conditions that made it right -- name order is correct in ls and find
   because those tools list everything, and the stability that justifies it disappears the
   moment the list is truncated. Common practice is evidence about expectations, not about
   correctness.

2. Truncate freely; never truncate silently. Bounding output is legitimate and often
   necessary. A reader who cannot tell it happened is not. Every bound states itself, says
   how much it dropped once the numbers are large, and is liftable by a flag named in the
   same breath.

Also fix the stale "Five Axes" heading, which the content axis made wrong.
