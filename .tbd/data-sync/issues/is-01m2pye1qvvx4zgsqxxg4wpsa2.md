---
type: is
id: is-01m2pye1qvvx4zgsqxxg4wpsa2
title: "P1.2.1: stored_state.rs: tier identities, EntryScope, serves_snapshot (Exact, Refuse), codecs"
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
    target: is-01m2pye22jsr1wv5rs1m7pwe4g
  - type: blocks
    target: is-01m2pye2d1y7k090q6k9d8mcc8
parent_id: is-01m2pmram44dgp78vm6xq4w7k7
hold: null
hold_until: null
created_at: 2026-09-17T05:46:34.107Z
updated_at: 2026-09-17T06:17:20.325Z
started_at: 2026-09-17T06:17:20.323Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 2: Store Identity, Equality Serve, and Per-Tier Writes", commit 1. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `stored_state.rs` (new): `EntryScope` (today's `ScanScope` without `type_rules_fingerprint`, `reducers_fingerprint`, and `ignore_rules_fingerprint`); `serves_snapshot(stored, wanted) -> Serves::{Exact, Refuse}`; `EntryTierIdentity { engine, scope: EntryScope, type_rules_fingerprint, reducers_fingerprint }`; `ControlTierIdentity::{NotObserved, Observed { limits }}`; `SnapshotIdentity { entries, controls }`; `ContentTierIdentity { entries, analysis, provenance }` with `serves` as equality; `entries_writable(&Index)`; `content_record_writable(&Index, &Path, &FileAnalysis)`; shared fixed-width codecs.
- `engine_contract.rs`, `scan.rs`: `ScanScope` (`engine_contract.rs:137-153`), `observes_controls()` (`:235-237`), `ScanConfig::scope()` (`scan.rs:325-335`). `ScanConfig::scope()` builds `EntryScope` and `ControlTierIdentity`; observation is read from `ControlTierIdentity` at `execution.rs:331`, `watch_session.rs:138`, `query/query_report.rs:976`, and `crates/fdu-py/src/lib.rs:375` and `:590`.
- The `EntryScope` split is settled here, before P1.2.2 writes snapshot format 5, so the format is written once.

**Tests**

- Unit tests for identity equality, `serves_snapshot` (`Exact` on equality, `Refuse` otherwise), the write predicates, and codec round trips.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
