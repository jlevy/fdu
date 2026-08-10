---
type: is
id: is-01kzn13r7sqzmfvrsjx317p7dt
title: Reuse verified base corpora across large benchmark trials
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzkzmsegmx4sfswka2084se6
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-10T05:07:48.086Z
updated_at: 2026-08-10T05:07:51.927Z
---
The first 1M exact-oracle cost-curve run spent more than twelve minutes in serial Python corpus construction/verification before launching the probe. Add a safe immutable base-corpus cache keyed by recipe hash, seed, target count, platform capabilities, and manifest schema; establish every trial from a clone/copy, verify its exact precondition, and preserve pristine restoration for churn. Prefer APFS clonefile/Linux reflink when capability-proven, fall back to bounded copy, never hardlink mutable files, serialize base creation, and exclude preparation time from component timing without hiding it from run diagnostics.
