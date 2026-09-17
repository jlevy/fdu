---
type: is
id: is-01m2pmra8yqrcxg27kc6ezg9vd
title: "Provenance model: per-tier source, freshness, observation time, completeness"
kind: task
status: open
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - design
dependencies:
  - type: blocks
    target: is-01m2pmrbxvnerxyhnjhrswy1ye
  - type: blocks
    target: is-01m2pmrcrmjrm62bm1x3mxwgvm
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
created_at: 2026-09-17T02:57:24.765Z
updated_at: 2026-09-17T02:57:27.315Z
---
Per tier: source, freshness, observation time, completeness, coverage, errors; composed into the report envelope.
Every route computes it: one-shot, open, Python Index, watch repaints (no hard-coded complete/errors/source),
opened reads. One meaning for scan_started_at. Content gains freshness.
