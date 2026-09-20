---
type: is
id: is-01kzyqkmswz9qha4knttpt9xzw
title: Expose bounded per-path content I/O diagnostics in reports
kind: bug
status: in_progress
priority: 2
version: 2
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
hold: null
hold_until: null
created_at: 2026-08-13T23:34:07.404Z
updated_at: 2026-09-20T05:33:22.477Z
started_at: 2026-09-20T05:33:22.477Z
---
FileAnalysis retains a bounded I/O error string, but grouped machine reports expose only aggregate coverage and the envelope receives only a generic category count. Add bounded per-path content diagnostics or explicitly revise the completed spec.
