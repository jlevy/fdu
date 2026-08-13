---
type: is
id: is-01kzn04cqdaaknww7941cbp7aw
title: Add opt-in dedicated Linux diagnostic collectors
kind: task
status: open
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzkzmsegmx4sfswka2084se6
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-10T04:50:40.492Z
updated_at: 2026-08-13T18:11:56.214Z
---
Add capability-negotiated, opt-in Linux byte-I/O, syscall-summary, perf-stat, and sampled-profile collectors for diagnosis and release evidence. Keep collector output separate from program stderr, record exact tool identity/configuration and overhead class, retain null-with-reason semantics, test parsers from committed fixtures, and never treat intrusive profile runs as headline latency samples.
