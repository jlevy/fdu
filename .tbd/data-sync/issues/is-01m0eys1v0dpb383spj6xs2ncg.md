---
type: is
id: is-01m0eys1v0dpb383spj6xs2ncg
title: Project experiment artifacts into a chart dataset
kind: task
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0eyrna93ewcb5nz1jh3gncc
created_at: 2026-08-20T06:47:15.551Z
updated_at: 2026-08-20T07:29:05.795Z
closed_at: 2026-08-20T07:29:05.794Z
close_reason: "benchmarks/realtree/timeline.py projects all 64 validated artifacts into a committed JSON dataset: per-experiment absolute medians per job and metric, paired change with interval, derived kept-variant, subject/family/scale, complexity, verdict, plus anchored series, calibration and totals. Paired and marginal readings are kept under separate keys so a chart cannot derive one from the other."
---
A module under benchmarks/realtree that reads the validated artifacts through softschema and emits one JSON dataset for the report: per-experiment absolute medians per job and metric, paired change with interval, derived kept-variant, subject and scale, complexity, and verdict. Every number read from the artifacts, never retyped, so the report cannot drift from the record.
