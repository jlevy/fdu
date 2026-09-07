---
type: is
id: is-01m1xbp9qy40ymd7wvyaw0ckp8
title: Remove ordered path-map work from public mutation preflight
kind: task
status: in_progress
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
hold: blocked
hold_until: null
created_at: 2026-09-07T07:18:00.700Z
updated_at: 2026-09-07T08:48:14.518Z
---
A counter-disabled call-tree profile of the unchanged 1a39be9 engine attributes 1936 of 2358 inclusive Index::apply samples to ancestry validation; StructuralOverlay::kind and upsert repeatedly compare path components in a BTreeMap. The overlay uses exact lookup, insertion and predicate retention, not ordered iteration. First characterize arbitrary operation order, repeated paths, directory replacement, subtree removal/recreation, controls, non-UTF-8 paths and atomic rejection through public contracts. Then compare a private standard HashMap overlay, without new dependencies, caller restrictions or changes to commit ordering. Preregister 12 interleaved pairs and 3 warmups for large and batched public mutations; require at least 3% median wall/component improvement with the paired 95% interval below zero, exact final-state and commit oracles, allocations/bytes and RSS within 1.05x, and cold/default/opened noninferiority. Re-profile to prove ancestry comparisons no longer dominate. Record negative evidence and revert the candidate if gates fail.

## Notes

Candidate ad52469 changes only the private exact-lookup overlay from BTreeMap to standard HashMap, with no new dependency, ordering restriction, unsafe code or public contract. Three new tests passed before and after the change; an omitted-pruning mutant failed the independent-model atomicity guard and was restored. Full make check and cross-lint passed on the candidate. Exp-102 completed the preregistered exploratory uncontrolled 12-pair/3-warmup mutation screen: large wall/component -49.78%/-77.84%, repeated batches -39.75%/-67.71%, all intervals below zero and all summaries/commit digests exact. Allocation events fall 1.78%/1.56%, reallocations are unchanged, bytes rise 2.39%/4.95% (the repeated case is close to the 5% ceiling), and peak RSS stays within 1.05. Counting all inclusive call-tree frames corrects the preliminary first-frame attribution to 1993/2391 ancestry samples before and 345/1192 after, about 83% to 29%. Raw pairs, provenance, counter outputs and profiles are retained with the experiment; ledger/report regenerated. Retain for confirmation, not final acceptance: generic all-zero context-switch qualification stays inconclusive, quiet mutation confirmation and one-shot/opened noninferiority remain open. Evidence validation and CI publication are in progress.
