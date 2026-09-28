---
type: is
id: is-01m3jrsanzdqxzn49jr5pqa3pa
title: Show delayed low-overhead throughput in progress
kind: task
status: in_progress
priority: 2
version: 3
labels: []
dependencies: []
created_at: 2026-09-28T01:06:36.350Z
updated_at: 2026-09-28T01:32:02.574Z
---

## Notes

Implemented delayed progress rates (>5 seconds), one-decimal seconds, 100 ms redraw and shared core throughput_rates used by progress and final perf. Both use grouped files/s and binary GiB/s; optional rates disappear before core facts at narrow widths and are absent without walk facts. 100 CLI tests and 3 terminal tests pass; independent arithmetic/style review found no defect. Full handoff gate in progress.
