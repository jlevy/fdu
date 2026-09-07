---
type: is
id: is-01m1xbp9qy40ymd7wvyaw0ckp8
title: Remove ordered path-map work from public mutation preflight
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T07:18:00.700Z
updated_at: 2026-09-07T07:19:51.613Z
---
A counter-disabled call-tree profile of the unchanged 1a39be9 engine attributes 1936 of 2358 inclusive Index::apply samples to ancestry validation; StructuralOverlay::kind and upsert repeatedly compare path components in a BTreeMap. The overlay uses exact lookup, insertion and predicate retention, not ordered iteration. First characterize arbitrary operation order, repeated paths, directory replacement, subtree removal/recreation, controls, non-UTF-8 paths and atomic rejection through public contracts. Then compare a private standard HashMap overlay, without new dependencies, caller restrictions or changes to commit ordering. Preregister 12 interleaved pairs and 3 warmups for large and batched public mutations; require at least 3% median wall/component improvement with the paired 95% interval below zero, exact final-state and commit oracles, allocations/bytes and RSS within 1.05x, and cold/default/opened noninferiority. Re-profile to prove ancestry comparisons no longer dominate. Record negative evidence and revert the candidate if gates fail.
