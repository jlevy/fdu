---
type: is
id: is-01m499cp52491kkx0cxzkqpmra
title: "PR #182 A1: page says dust measured in eleven runs; it was nine"
kind: bug
status: in_progress
priority: 2
version: 2
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m499cf1q15te09v5zkpvdxd2
hold: null
hold_until: null
created_at: 2026-10-06T19:00:05.408Z
updated_at: 2026-10-06T19:00:10.932Z
started_at: 2026-10-06T19:00:10.931Z
---
Low. explorations/benchmarks/realtree/report_html.py:1941 and docs/project/reports/report-2026-08-20-fdu-performance-evidence.md:1385-1386. Eleven dust calibrations come from nine run artifacts. Coordinator choice: compute the range and distinct-run count in _section_relative from dataset['calibration']; fix the evidence report to nine. PR #182, review https://github.com/jlevy/fdu/pull/182#issuecomment-6022824692
