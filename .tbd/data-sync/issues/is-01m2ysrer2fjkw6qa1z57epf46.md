---
type: is
id: is-01m2ysrer2fjkw6qa1z57epf46
title: Concurrent reconciliation can erase newer omitted failures
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T06:58:47.680Z
updated_at: 2026-09-20T06:58:47.680Z
---
Two public full-root refreshes can overlap because IndexHandle locks only begin/finish boundaries. Baseline subtraction by an older closer can erase omission counts produced by a newer pass and permit false Complete coverage. Track omitted failures with bounded epoch ownership and preserve newer-pass omissions.
