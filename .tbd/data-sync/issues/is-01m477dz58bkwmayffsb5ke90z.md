---
type: is
id: is-01m477dz58bkwmayffsb5ke90z
title: Review all notes and tips for concision and consolidate overlapping lines
kind: task
status: closed
priority: 1
version: 4
labels: []
dependencies:
  - type: blocks
    target: is-01m477dzjmz4414feb9mxpnnr4
  - type: blocks
    target: is-01m47846395xp250085ae705pe
parent_id: is-01m477dy519nmp2vvdjtg0n3z1
created_at: 2026-10-05T23:47:21.383Z
updated_at: 2026-10-06T01:12:00.250Z
closed_at: 2026-10-06T01:12:00.249Z
close_reason: "Notes/tips consolidated: totals, display limits, one runnable show-more tip; flat notes shortened. Analysis-axis notes are rewritten under fdu-b8tr."
resolution: null
duplicate_of: null
---
Inventory every note:/tip:/warn: producer (report_epilogue.rs, query_report.rs, report_format.rs, execution.rs, content_analysis.rs, cli.rs). Shorten each to the fewest words that keep its facts, and merge lines that carry overlapping information (e.g. the tree remainder and display-limit notes, the share/depth tips) into fewer lines. Keep the facts needed to debug; keep categories distinct (facts vs remedies). Update the output design doc examples, goldens, and Python parity.
