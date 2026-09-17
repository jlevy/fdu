---
type: is
id: is-01m2pye2d1y7k090q6k9d8mcc8
title: "P1.2.3: Content sidecar format 5 with ContentTierIdentity and identify_sidecar"
kind: task
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye2qa2z80jqz73hnp5y2h
parent_id: is-01m2pmram44dgp78vm6xq4w7k7
created_at: 2026-09-17T05:46:34.784Z
updated_at: 2026-09-17T05:46:51.347Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 2: Store Identity, Equality Serve, and Per-Tier Writes", commit 3. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `content/content_cache.rs`: `FORMAT_VERSION` (`:23`) to 5; `save_content_cache` (`:56-105`) writes the engine fingerprint and `ContentTierIdentity`; `parse` (`:218-282`) reads them.
- `content/content_cache.rs`: add `identify_sidecar(path)`, mirroring `snapshot::identify` (`:515-529`); `content_sidecar_bytes` (`:182-202`) stays magic-only so older sidecars are still reclaimed.

**Tests**

- Recheck `corruption_is_a_clean_miss` (`:721`) offsets.
- Add `a_sidecar_from_another_engine_or_scope_is_a_clean_miss`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
