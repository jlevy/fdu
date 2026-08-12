---
type: is
id: is-01kzpvt22bex8ed6d155y014py
title: Profile and optimize compatible-snapshot real-tree revalidation
kind: task
status: in_progress
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzpvt29hvtsrg1pyrq20awxa
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
child_order_hints:
  - is-01kztxsn4qpkq1y652qtwb6vta
created_at: 2026-08-10T22:13:36.458Z
updated_at: 2026-08-12T12:30:20.391Z
---
Using the same immutable real-tree subject, profile compatible-unchanged open/revalidation, snapshot load, first listing, full completion, and verified-warm repetitions. Preserve the rule that directory fingerprints may skip only name-set discovery, never child truth checks. Use exact pre/post oracles, interleaved paired trials, resource evidence, and one commit per accepted optimization; reject complexity without stable end-to-end gains.

## Notes

Warm loop through exp-026: exp-002 parallel sweep rejected at -2.6%; H14/H5/H10/H32 constant reductions landed; H53 now reuses the audited macOS getattrlistbulk reader in direct/shared/scoped full reconciliation. Final exact-binary wall: -18.97% at 60k and -34.39% at 720k; large system CPU -53.97%, RSS neutral. Current 60k warm open is about 500 ms versus about 296 ms cold, so the cache remains structurally slower without journal scoping/persisted roll-ups. Post-change profile removes fstatat/getdirentries; directory open is 30.27% and getattrlistbulk 16.61%.
