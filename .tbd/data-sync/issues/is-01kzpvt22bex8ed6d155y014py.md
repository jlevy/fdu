---
type: is
id: is-01kzpvt22bex8ed6d155y014py
title: Profile and optimize compatible-snapshot real-tree revalidation
kind: task
status: closed
priority: 1
version: 13
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzpvt29hvtsrg1pyrq20awxa
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
child_order_hints:
  - is-01kztxsn4qpkq1y652qtwb6vta
  - is-01kzv0dfwab6whteads2kzy2f9
  - is-01kzv21f0c1508pd22gxrncxy3
created_at: 2026-08-10T22:13:36.458Z
updated_at: 2026-08-23T02:11:33.033Z
closed_at: 2026-08-12T13:24:49.962Z
close_reason: Compatible-snapshot full-revalidation loop complete through exp-030/032. H12 bounded four-worker immutable-baseline waves improve warm wall 30.25% at 60k and 59.53% at 720k over exp-026; cumulative warm wall -54.26% versus b565882. Snapshot load now owns most of the 60k warm-vs-cold gap and remains tracked separately by fdu-1vd0; FSEvents orchestration is a separate scoped-cache feature.
---
Using the same immutable real-tree subject, profile compatible-unchanged open/revalidation, snapshot load, first listing, full completion, and verified-warm repetitions. Preserve the rule that directory fingerprints may skip only name-set discovery, never child truth checks. Use exact pre/post oracles, interleaved paired trials, resource evidence, and one commit per accepted optimization; reject complexity without stable end-to-end gains.

## Notes

Warm loop through exp-030: H53 bulk-backed reconciliation landed in exp-026, then H12 bounded immutable-baseline waves removed exact no-ops before the consumer. Exact-binary warm wall improved another 30.25% at 60k and 59.53% at 720k; component -50.31%/-72.55%, 60k RSS +3.29%, large RSS -0.99%. Four workers beat six significantly at 60k; six was unclear at 720k. Final exp-032 cumulative warm wall is -51.99% versus b565882. Snapshot bulk-load/persisted roll-ups now own most of H9; FSEvents orchestration remains the asymptotic path.
