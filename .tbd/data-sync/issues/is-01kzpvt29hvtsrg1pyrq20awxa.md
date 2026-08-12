---
type: is
id: is-01kzpvt29hvtsrg1pyrq20awxa
title: Publish the real-tree optimization decision ledger
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-10T22:13:36.688Z
updated_at: 2026-08-12T13:24:50.271Z
---
After the cold and warm loops, validate accepted changes on deterministic 10k/100k/500k corpora plus at least one tens-of-thousands-file real checkout. Record raw evidence hashes, host/build identity, medians and dispersion, profiles, accepted and rejected experiments, complexity rationale, and remaining bottlenecks. Update the plan and PR after every accepted commit and make no product speed claim until the dedicated-host comparator matrix passes.

## Notes

Decision ledger now has 32 validated experiments and exp-032 is the exact cumulative anchor: cold index -53.59%, producer -57.87%, snapshot save -51.33%, warm revalidation -54.26%, snapshot load -35.25% versus b565882 on the 60k real tree. 120k and 720k scale evidence exists for the relevant cold/warm changes. Cold and warm optimization loops are closed; deterministic 10k/100k/500k corpus validation and the dedicated-host comparator matrix remain before any product claim.
