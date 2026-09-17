---
type: is
id: is-01m2pye2qa2z80jqz73hnp5y2h
title: "P1.2.4: Serve content by identity equality; delete satisfies and contains"
kind: task
status: in_progress
priority: 0
version: 7
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: claude-code@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye31mfaxjr62p7s6jdk4x
  - type: blocks
    target: is-01m2pye4c1h1kfcnsq59yfsnbt
  - type: blocks
    target: is-01m2pye7bt83a2erfcav6hhkfc
  - type: blocks
    target: is-01m2phvqccpjdrbbsr5y3kcydr
parent_id: is-01m2pmram44dgp78vm6xq4w7k7
hold: null
hold_until: null
created_at: 2026-09-17T05:46:35.114Z
updated_at: 2026-09-17T16:27:28.338Z
started_at: 2026-09-17T15:02:00.779Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 2: Store Identity, Equality Serve, and Per-Tier Writes", commit 4. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `content/content_model.rs`: delete `ContentProvenance::satisfies` (`:266-273`) and `AnalysisSet::contains` (`:92-99`).
- `content/content_index.rs`: `ContentIndex` (`:166-172`) holds `identity: Option<ContentTierIdentity>` instead of `profile`/`provenance` (`:186-193`); `prepare` (`:252-272`) clears on any inequality; `commit` (`:209-217`) refuses a record of another identity instead of calling `prepare`.
- `index.rs`: `prepare_content_analysis` (`:3390-3398`) builds the identity from the index's scope and registry; `pending_analysis_candidates` (`:3434-3453`) compares fingerprint and identity; `apply_analysis` (`:3457-3474`) returns `Stale` when `commit` refuses; add `Index::content_set() -> AnalysisSet`.
- `content/content_cache.rs`: `load_content_cache` (`:109-175`) compares identity by equality.
- `lib.rs`: `load_content` (`:719-724`) passes the content identity; the cache-only check (`:541-551`) keeps its count, which now means complete.
- Call sites: `satisfies` at `content/content_cache.rs:235`, `content/content_index.rs:262`, `index.rs:3445`; `load_content_cache` at `lib.rs:723`; `analyze_index`, whose behavior changes, at `lib.rs:574` and `:615`, `crates/fdu-py/src/lib.rs:466`, `examples/perf_probe.rs:501` and `:550`, and the `content/content_cache.rs` tests at `:634` and `:650`.
- Registry: clear `content-containment` and `mixed-records` after a full Linux run (only the full matrix shows a class is empty).

**Tests**

- `content/content_cache.rs`: turn the containment tests (`:661`, `:682`, `:701`) into `a_wider_sidecar_is_a_clean_miss_for_a_narrower_request`, `another_analyzer_set_is_a_clean_miss`, and `a_different_analyzer_set_replaces_the_sidecar`; add `an_analyzer_version_change_invalidates_records`.
- `content/content_model.rs`: delete `containment_is_reflexive_and_ordered_by_membership` (`:575-585`).
- `content/content_index.rs`: add `prepare_clears_on_any_identity_change` and `commit_refuses_a_record_of_another_identity`.
- `lib.rs`: recheck `content_sidecar_skips_unchanged_reads_and_serves_cache_only` (`:1286`) and `cache_only_analysis_fails_closed_without_its_sidecar` (`:1362`).
- Goldens: `cli-cache` and `cli-content` should not change.

**Done when**

- The registry holds only the `projection-route` and `unverified-subtree` classes; fdu-gija's reproduction answers the same warm and cold.
- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Risk: alternating analyzer sets re-read files and replace the sidecar, giving back the gain #37 measured until content subset projection lands (a scope deferral).

## Notes

Layer 3 of the core-models stack: branch claude/core-models-3-equality-serve, PR https://github.com/jlevy/fdu/pull/82 (stack #80). Close when the layer merges.
