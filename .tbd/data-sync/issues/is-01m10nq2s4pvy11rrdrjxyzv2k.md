---
type: is
id: is-01m10nq2s4pvy11rrdrjxyzv2k
title: Expose immutable Python opened-root models, stubs, and wrapper
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nq345g2dt9hqmxq7kyvrg
parent_id: is-01m0y1sf2nph021wtx28p8ahxh
created_at: 2026-08-27T03:55:13.571Z
updated_at: 2026-08-27T05:35:13.172Z
closed_at: 2026-08-27T05:35:13.172Z
close_reason: Implemented and verified the direct opened-root Python surface at 0583a1a/fa85812. Full make check, cross-lint, installed wheel/sdist lifecycle and typing, CLI/Python parity, MSRV, and the complete GitHub Actions matrix all pass. The standalone CLI raw stripped size is unchanged and its golden corpus remains green; no runtime dependency was added.
resolution: null
duplicate_of: null
---
Add the cohesive public python/fdu/opened.py namespace with immutable fdu-native models and the direct OpenedIndex wrapper, and update _native.pyi while preserving every existing top-level one-shot name. Validate paths and request bounds once at the public boundary while keeping engine state, paging, lifecycle, and aggregation decisions native. The package is a reusable Python peer of fdu-core, not a MetaBrowser SDK or a second command-line implementation.
