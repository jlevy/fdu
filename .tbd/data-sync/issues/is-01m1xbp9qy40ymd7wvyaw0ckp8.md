---
type: is
id: is-01m1xbp9qy40ymd7wvyaw0ckp8
title: Remove ordered path-map work from public mutation preflight
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T07:18:00.700Z
updated_at: 2026-09-07T07:39:30.681Z
---
A counter-disabled call-tree profile of the unchanged 1a39be9 engine attributes 1936 of 2358 inclusive Index::apply samples to ancestry validation; StructuralOverlay::kind and upsert repeatedly compare path components in a BTreeMap. The overlay uses exact lookup, insertion and predicate retention, not ordered iteration. First characterize arbitrary operation order, repeated paths, directory replacement, subtree removal/recreation, controls, non-UTF-8 paths and atomic rejection through public contracts. Then compare a private standard HashMap overlay, without new dependencies, caller restrictions or changes to commit ordering. Preregister 12 interleaved pairs and 3 warmups for large and batched public mutations; require at least 3% median wall/component improvement with the paired 95% interval below zero, exact final-state and commit oracles, allocations/bytes and RSS within 1.05x, and cold/default/opened noninferiority. Re-profile to prove ancestry comparisons no longer dominate. Record negative evidence and revert the candidate if gates fail.

## Notes

Before changing storage, new mixed-batch and remove/recreate public tests passed against the independent model, and a retained/transient control-pruning test passed. An isolated injected mutant that omitted overlay subtree pruning made the new atomicity test fail as intended; it was restored immediately. The candidate changes only the private entries container from BTreeMap to standard HashMap, retaining lookup, removal and operation ordering. The model and control tests are now being rerun against that candidate. Host pressure remains unsuitable for final quiet timing; no thresholds or final acceptance scope have been relaxed.
