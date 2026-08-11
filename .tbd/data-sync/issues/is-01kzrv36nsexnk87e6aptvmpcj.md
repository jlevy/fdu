---
type: is
id: is-01kzrv36nsexnk87e6aptvmpcj
title: Promote persisted roll-ups and lazy open for the browser use case
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
created_at: 2026-08-11T16:39:36.376Z
updated_at: 2026-08-11T16:47:54.211Z
---
Re-prioritization, not new design: research H33 (persist per-directory reducer state), H34 (bulk arena load), H16 (answer a query from roll-ups without materializing the index) and H35 (block format with tail index and lazy decompression, bead fdu-1vd0) were sequenced low when the driver was a one-shot CLI. The interactive-browser use case (research-2026-08-11-interactive-browser-use-case.md) makes them the single biggest lever: a browser's second open of a home folder needs first paint in under 100 ms, and at today's ~2 us/record a full load of 5.4M entries is ~11 s. Sizes are recursive so only a cache can answer at T0, and only a format that reads the top level without materializing every record can answer fast. Also enables T2: a browser shows a few hundred entries at a time and should load blocks for the directory being viewed rather than the whole tree.
