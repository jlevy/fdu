---
type: is
id: is-01m10nq2s4pvy11rrdrjxyzv2k
title: Expose immutable Python opened-root models, stubs, and wrapper
kind: task
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nq345g2dt9hqmxq7kyvrg
parent_id: is-01m0y1sf2nph021wtx28p8ahxh
created_at: 2026-08-27T03:55:13.571Z
updated_at: 2026-08-27T05:21:57.125Z
---
Add the cohesive public python/fdu/opened.py namespace with immutable fdu-native models and the direct OpenedIndex wrapper, and update _native.pyi while preserving every existing top-level one-shot name. Validate paths and request bounds once at the public boundary while keeping engine state, paging, lifecycle, and aggregation decisions native. The package is a reusable Python peer of fdu-core, not a MetaBrowser SDK or a second command-line implementation.
