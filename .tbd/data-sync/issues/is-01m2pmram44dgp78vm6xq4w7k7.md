---
type: is
id: is-01m2pmram44dgp78vm6xq4w7k7
title: "Phase 1 item 2: store identity, equality serve, and per-tier writes"
kind: epic
status: open
priority: 0
version: 19
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - design
  - cache
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01m2pye1qvvx4zgsqxxg4wpsa2
  - is-01m2pye22jsr1wv5rs1m7pwe4g
  - is-01m2pye2d1y7k090q6k9d8mcc8
  - is-01m2pye2qa2z80jqz73hnp5y2h
  - is-01m2pye31mfaxjr62p7s6jdk4x
  - is-01m2pye3bwkv6049gkcyh3rrcp
  - is-01m2qygtb7z3mehzrgzwz3mtx3
  - is-01m2qz74mbthxxy2eky1qgb7eh
  - is-01m2rgb3631yn1waftyaj31wca
created_at: 2026-09-17T02:57:25.124Z
updated_at: 2026-09-17T20:18:46.072Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 2: Store Identity, Equality Serve, and Per-Tier Writes", moved here when the item became beads (PR #78 at `e52383d4`; locators verified at `5f2d36d`). The commits are this bead's children, P1.2.1 to P1.2.6; their blockers carry the ordering, so this bead only groups them and closes when they do.

| File | Function or type | Change |
| --- | --- | --- |
| `stored_state.rs` (new) | `EntryScope` (today’s `ScanScope` without `type_rules_fingerprint`, `reducers_fingerprint`, and `ignore_rules_fingerprint`), `serves_snapshot(stored, wanted) -> Serves::{Exact, Refuse}`, `EntryTierIdentity { engine, scope: EntryScope, type_rules_fingerprint, reducers_fingerprint }`, `ControlTierIdentity::{NotObserved, Observed { limits }}`, `SnapshotIdentity { entries, controls }`, `ContentTierIdentity { entries, analysis, provenance }` with `serves` as equality, `entries_writable(&Index)`, `content_record_writable(&Index, &Path, &FileAnalysis)`, and shared fixed-width codecs | Add |
| `snapshot.rs` | `FORMAT_VERSION` (`:64`); `save` (`:216-281`); `put_scope`/`read_scope` (`:924-969`); `read_controls` limits (`:788-791`); `parse_header_fields` (`:599-611`); `parse_stream` (`:614-708`) | Format 5 with header identities and the verifying pass’s start (`verified_started_at_ns`), which `save` writes; today’s `captured_at_ns` is the file’s modification time (`snapshot.rs:369-375`); the save guard becomes `entries_writable`; control limits move into the header and a disagreeing table is refused. `identify_prologue` (`:532-566`) keeps its offsets, so format 4 reads as `OlderFormat` |
| `content/content_cache.rs` | `FORMAT_VERSION` (`:23`); `save_content_cache` (`:56-105`); `load_content_cache` (`:109-175`); `parse` (`:218-282`) | Format 5 with the engine fingerprint and `ContentTierIdentity`; the save filter (`:71-81`) becomes `content_record_writable`; loading compares identity by equality |
| `content/content_cache.rs` | `identify_sidecar(path)` | Add, mirroring `snapshot::identify` (`:515-529`); `content_sidecar_bytes` (`:182-202`) stays magic-only so older sidecars are still reclaimed |
| `engine_contract.rs`, `scan.rs` | `ScanScope` (`engine_contract.rs:137-153`), `observes_controls()` (`:235-237`), `ScanConfig::scope()` (`scan.rs:325-335`) | `ScanConfig::scope()` builds `EntryScope` and `ControlTierIdentity`; observation is read from `ControlTierIdentity` at `execution.rs:331`, `watch_session.rs:138`, `query/query_report.rs:976`, and `crates/fdu-py/src/lib.rs:375` and `:590` |
| `content/content_model.rs` | `ContentProvenance::satisfies` (`:266-273`), `AnalysisSet::contains` (`:92-99`) | Delete |
| `content/content_index.rs` | `ContentIndex` (`:166-172`), `profile`/`provenance` (`:186-193`), `prepare` (`:252-272`), `commit` (`:209-217`) | Hold `identity: Option<ContentTierIdentity>`; `prepare` clears on any inequality; `commit` refuses a record of another identity instead of calling `prepare` |
| `index.rs` | `prepare_content_analysis` (`:3390-3398`), `pending_analysis_candidates` (`:3434-3453`), `apply_analysis` (`:3457-3474`) | Build the identity from the index’s scope and registry; pending compares fingerprint and identity; `apply_analysis` returns `Stale` when `commit` refuses; add `content_set()` |
| `lib.rs` | `load_content` (`:719-724`), cache-only check (`:541-551`), `SaveTargets` (`:636-653`), `cold_scan_save_targets_with` (`:699-717`), `spawn_save` (`:730-777`) | Pass the content identity; keep the cache-only count, which now means complete; drop the joint completeness gate (`:736-737`) for per-tier rules; write the sidecar after a partial scan only when a snapshot of the same entry identity is already stored, so a partial run under another identity never evicts the sidecar that pairs with the stored snapshot |
| `cache.rs` | `SnapshotInfo` (`:245-253`), `CacheStatus` (`:43-53`), `status_at` (`:398-400`) | `SnapshotInfo.identity`; `CacheStatus.content` from `identify_sidecar`; pairing, `clear_cache`, and the `OrphanedContent` rule stay magic-based |
| `report_format.rs`, `crates/fdu-py/src/lib.rs`, `crates/fdu-py/python/fdu/_models.py` | `CACHE_SCHEMA` (`:70`), `render_cache_status` (`:1610-1714`); `cache_status_dict` (`:1484-1523`); `CacheStatus` model (`:817-836`) | `fdu.cache/2` with identities, coordinated with the answer model |
| Documentation | `fdu.cache/2` and formats 5 in `docs/project/guides/cache-design.md`, `docs/project/architecture/fdu-surface-architecture.md`, `docs/project/architecture/fdu-engine-architecture.md`, `docs/project/release-notes/0.1.0.md`, and `docs/project/guides/release-process.md` | Update |

**Call sites:** `SnapshotInfo` constructions at `snapshot.rs:609`, `cache.rs:1102`,
`report_format.rs:1945`, and its readers at `crates/fdu-py/src/lib.rs:1514-1515`;
`satisfies` at `content/content_cache.rs:235`, `content/content_index.rs:262`,
`index.rs:3445`; `save_content_cache` at `lib.rs:768`; `load_content_cache` at
`lib.rs:723`; `analyze_index`, whose behavior changes, at `lib.rs:574` and `:615`,
`crates/fdu-py/src/lib.rs:466`, `examples/perf_probe.rs:501` and `:550`, and the
`content/content_cache.rs` tests at `:634` and `:650`.

**Tests:**
- `content/content_cache.rs`: turn the containment tests (`:661`, `:682`, `:701`) into
  `a_wider_sidecar_is_a_clean_miss_for_a_narrower_request`,
  `another_analyzer_set_is_a_clean_miss`, and
  `a_different_analyzer_set_replaces_the_sidecar`; recheck `corruption_is_a_clean_miss`
  (`:721`) offsets; add `a_sidecar_from_another_engine_or_scope_is_a_clean_miss`,
  `an_analyzer_version_change_invalidates_records`, and
  `records_under_an_unverified_subtree_are_not_written`.
- `content/content_model.rs`: delete
  `containment_is_reflexive_and_ordered_by_membership` (`:575-585`).
  `content/content_index.rs`: add `prepare_clears_on_any_identity_change` and
  `commit_refuses_a_record_of_another_identity`.
- `snapshot.rs`: update `semantic_scan_scope_round_trips` (`:2435`) and
  `control_limits_that_disagree_with_the_scope_are_refused_at_save_and_load` (`:1639`);
  add `a_v4_snapshot_is_older_format` and
  `controls_on_and_off_snapshots_share_the_entry_identity`.
- `lib.rs`: add `a_partial_scan_writes_verified_content_but_no_snapshot`; recheck
  `content_sidecar_skips_unchanged_reads_and_serves_cache_only` (`:1286`) and
  `cache_only_analysis_fails_closed_without_its_sidecar` (`:1362`). `cache.rs`: add
  `a_stale_sidecar_beside_a_current_snapshot_is_labelled_and_cleared`.
- Goldens: `cli-lifecycle` cache status in JSON and YAML and the stale listing change
  for `fdu.cache/2`; `cli-surface`’s `--docs` text changes; `cli-cache` and
  `cli-content` should not change.
  The parity artifact is re-recorded by CI.

**Commits:**
1. `stored_state.rs` types, `EntryScope`, `serves_snapshot` with `Exact` and `Refuse`,
   and codecs, with unit tests.
2. Snapshot format 5, including `verified_started_at_ns`.
3. Sidecar format 5 and `identify_sidecar`.
4. Equality serve; delete `satisfies` and `contains`; clear two registry classes after a
   full Linux run.
5. Per-tier write rules.
6. Cache status identities, bindings, and goldens.

**Risks:** alternating analyzer sets re-read files and replace the sidecar, giving back
the gain #37 measured until subset projection lands; two sidecar format bumps if
per-analyzer records land separately.

## Original scope (before the implementation detail)

One contract for entries, .gitignore control state, classification, and content: identity (engine fingerprint plus
exactly the request parts that shape the tier), one per-item fingerprint (size, allocated, mtime, ctime, inode,
device; source bytes where content-derived), serves and project (equality by default), storage keyed by identity
with eviction as a miss, per-tier provenance. New snapshot and content-store formats; old files read as stale.

## Notes

2026-09-17 (PR #78 review): Phase 1 item 2. Scope reduced: store headers carry every tier's identity (sidecar gains scope, engine fingerprint, analyzer versions and options), content served by equality of analyzer sets, per-tier write rule (entries only after a complete scan; control state and content records with verified items). Stores keyed by identity are a deferral (fdu-w3l5).
