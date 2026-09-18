---
type: is
id: is-01m2thn65yev0rqddsdbt7mra9
title: "CHANGELOG.md watch bullet conflicts between PR #85 and PR #87"
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-18T15:20:14.270Z
updated_at: 2026-09-18T15:20:14.270Z
---
Both branches rewrite the same CHANGELOG.md sentence ('A watch is metadata-only and refuses --analyze.') in different words, so whichever merges second conflicts. Verified symmetric by merge simulation in both orders; every other PR pair among #84-#88 merges clean. Resolution: keep PR #87's wording, which is a strict content superset - it states the per-surface refusal (CLI --analyze with --watch; Rust Session and Python Index.watch() refusing an index opened with content analysis, as Error::UnsupportedScanConfig or InvalidArgumentError) and then 'The request model also refuses a narrowed scan scope and cache-only', which covers everything PR #85's shorter version says. No other CHANGELOG hunk conflicts; PR #85's 'Request model' Added bullet merges clean.
