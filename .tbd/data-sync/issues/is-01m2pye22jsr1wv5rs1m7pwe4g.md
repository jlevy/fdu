---
type: is
id: is-01m2pye22jsr1wv5rs1m7pwe4g
title: "P1.2.2: Snapshot format 5 with tier identities and verified_started_at_ns"
kind: task
status: open
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye2qa2z80jqz73hnp5y2h
  - type: blocks
    target: is-01m2pye6pak55wktt2d0ge1je2
  - type: blocks
    target: is-01m2pyeeppbcbq3hfepwjcthyw
parent_id: is-01m2pmram44dgp78vm6xq4w7k7
created_at: 2026-09-17T05:46:34.449Z
updated_at: 2026-09-17T05:47:01.064Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 2: Store Identity, Equality Serve, and Per-Tier Writes", commit 2. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `snapshot.rs`: `FORMAT_VERSION` (`:64`) to 5; `save` (`:216-281`) writes `SnapshotIdentity` in the header and the verifying pass's start as `verified_started_at_ns` (today's `captured_at_ns` is the file's modification time, `snapshot.rs:369-375`); `put_scope`/`read_scope` (`:924-969`); `parse_header_fields` (`:599-611`); `parse_stream` (`:614-708`).
- Control limits (`read_controls`, `:788-791`) move into the header; a disagreeing table is refused.
- `identify_prologue` (`:532-566`) keeps its offsets, so format 4 reads as `OlderFormat`.

**Tests**

- Update `semantic_scan_scope_round_trips` (`:2435`) and `control_limits_that_disagree_with_the_scope_are_refused_at_save_and_load` (`:1639`).
- Add `a_v4_snapshot_is_older_format` and `controls_on_and_off_snapshots_share_the_entry_identity`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
