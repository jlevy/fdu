---
type: is
id: is-01m3g3en7kwzw3342way3t4c85
title: Design multi-root disk-pressure history with hourly and daily comparisons
kind: feature
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
created_at: 2026-09-27T00:15:17.736Z
updated_at: 2026-09-27T00:15:17.736Z
---
Add an opt-in disk-history workflow alongside ordinary rollups and cache-assisted content analysis. Track Home, Applications, and canonical temporary roots without aliases or overlapping totals; retain immutable checkpoints for actual hour/day boundaries, share per-volume free-space samples, rank net growth and shrinkage, expose partial/mixed-age coverage and unexplained volume changes, and bound low-space refresh/persistence. FSEvents discovers changed scopes but does not reconstruct historical sizes or process causality. No mandatory daemon; optional scheduled captures preserve time boundaries. Implement through core and mirror CLI/Python after checkpoint and replay contracts.
