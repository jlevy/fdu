---
type: is
id: is-01m2pmra8yqrcxg27kc6ezg9vd
title: "Provenance model: per-tier source, freshness, observation time, completeness"
kind: task
status: open
priority: 0
version: 4
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
updated_at: 2026-09-17T03:38:54.449Z
---
Per tier: source, freshness, observation time, completeness, coverage, errors; composed into the report envelope.
Every route computes it: one-shot, open, Python Index, watch repaints (no hard-coded complete/errors/source),
opened reads. One meaning for scan_started_at. Content gains freshness.

## Notes

2026-09-17 (PR #78 review): Phase 1 item 4. Split tree status (complete, errors, coverage; part of the answer, compared by the invariant) from provenance (source, freshness, timestamps; excluded). Compute both on every route from the index, generalizing opened/read.rs:254-263; one meaning for scan_started_at; never serve retained facts under an unverified subtree (partial answers contain exactly the verified part).
