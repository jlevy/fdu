---
type: is
id: is-01kzs5162sbb3w464j2p1mgs6k
title: Provenance types and per-entry source byte
kind: task
status: closed
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzs518dfemmfeypev5s384j8
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:33:15.982Z
updated_at: 2026-08-11T19:41:39.244Z
closed_at: 2026-08-11T19:41:39.243Z
close_reason: Landed in 3e4c62d. Provenance/Source/Status in types.rs, exported from lib.rs; Source stored as one byte per entry in existing padding per the plan's storage design; timestamps held once on the index; Provenance built on demand by Index::provenance. Tests cover source ordering, commutative weakest-of-each combination, unknown-timestamp handling, and is_verified.
---
Foundation everything else reads. Add Provenance { source: Source, observed_at: SystemTime, status: Status } as a VIEW type in types.rs - constructed on demand, never a stored field. Source = Scanned | Revalidated | JournalConfirmed | Cached, ordered weakest-last. Status = Complete | Partial, ordered and #[non_exhaustive] so the anticipated Truncated (cap or boundary hit; a floor that will NOT grow - metabrowser's existing truncated state) and Errored (some children unreadable; exact for what was visible) slot in without touching consumers that only ask settled-or-not. STORAGE per the plan's design section: one Source byte on Entry, fitting existing padding beside kind and ext_id - NOT a 24-byte struct per entry, because entries already cost ~493 B each and the frontier research wants that near 50 B. Timestamps live once per index, not per entry. Unit tests for the orderings.
