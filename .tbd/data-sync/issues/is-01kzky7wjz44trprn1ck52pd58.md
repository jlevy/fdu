---
type: is
id: is-01kzky7wjz44trprn1ck52pd58
title: Add a deterministic reference model for index and delta transitions
kind: task
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - testing
  - correctness
dependencies:
  - type: blocks
    target: is-01kzg49sfhtxshw3senkhjmc24
  - type: blocks
    target: is-01kzg49sswr78gpjykxctbe6c7
  - type: blocks
    target: is-01m0y1sawtthrp0bq2agcv07f8
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T18:58:23.454Z
updated_at: 2026-08-26T03:28:53.538Z
---
The index has strong example tests but no independent model for long operation sequences. Build a small canonical tree model that recomputes state and roll-ups from scratch. Generate fixed-seed sequences covering out-of-order upserts, removals, kind replacement, no-ops, invalidation, delayed conditional observations, ABA, journal truncation, and non-Unicode names. Compare paths, kinds, attributes, roll-ups, clock/effective-delta behavior, freshness, and pending invalidations after every step. Print the seed and full operation trace on failure and retain minimized discoveries as focused regressions. Prefer a dependency-free generator unless a reviewed property-test dependency proves that shrinking materially improves diagnosis.
