---
type: is
id: is-01m2pye3bwkv6049gkcyh3rrcp
title: "P1.2.6: Cache status carries tier identities (fdu.cache/2), with bindings, goldens, and docs"
kind: task
status: in_progress
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: claude-code@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyeb3yz78d3d94dv78dv4g
parent_id: is-01m2pmram44dgp78vm6xq4w7k7
hold: null
hold_until: null
created_at: 2026-09-17T05:46:35.771Z
updated_at: 2026-09-17T16:27:30.036Z
started_at: 2026-09-17T15:02:01.502Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 2: Store Identity, Equality Serve, and Per-Tier Writes", commit 6. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `cache.rs`: `SnapshotInfo` (`:245-253`) gains `identity`; `CacheStatus` (`:43-53`) gains `content` from `identify_sidecar`; `status_at` (`:398-400`). Pairing, `clear_cache`, and the `OrphanedContent` rule stay magic-based.
- `report_format.rs`: `CACHE_SCHEMA` (`:70`) to `fdu.cache/2`; `render_cache_status` (`:1610-1714`). Coordinate with P2.2.6, which later moves cache status onto the answer walks.
- `crates/fdu-py/src/lib.rs`: `cache_status_dict` (`:1484-1523`); `crates/fdu-py/python/fdu/_models.py`: `CacheStatus` model (`:817-836`).
- Call sites: `SnapshotInfo` constructions at `snapshot.rs:609`, `cache.rs:1102`, `report_format.rs:1945`; readers at `crates/fdu-py/src/lib.rs:1514-1515`.
- Documentation: `fdu.cache/2` and formats 5 in `docs/project/guides/cache-design.md`, `docs/project/architecture/fdu-surface-architecture.md`, `docs/project/architecture/fdu-engine-architecture.md`, `docs/project/release-notes/0.1.0.md`, and `docs/project/guides/release-process.md`.

**Tests**

- `cache.rs`: add `a_stale_sidecar_beside_a_current_snapshot_is_labelled_and_cleared`.
- Goldens: `cli-lifecycle` cache status in JSON and YAML and the stale listing change for `fdu.cache/2`; `cli-surface`'s `--docs` text changes. The parity artifact is re-recorded by CI.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

## Notes

Layer 3 of the core-models stack: branch claude/core-models-3-equality-serve, PR https://github.com/jlevy/fdu/pull/82 (stack #80). Close when the layer merges.
