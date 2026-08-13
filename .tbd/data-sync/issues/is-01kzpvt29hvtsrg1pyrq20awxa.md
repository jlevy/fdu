---
type: is
id: is-01kzpvt29hvtsrg1pyrq20awxa
title: Publish the real-tree optimization decision ledger
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-10T22:13:36.688Z
updated_at: 2026-08-13T18:28:25.720Z
closed_at: 2026-08-13T18:28:25.719Z
close_reason: "Complete for the macOS landing: 47 validated artifacts generate the decision ledger, covering real 60k, 720k, 901k, and 1.0M subjects plus accepted, rejected, superseded, and in-progress results. Controlled Linux extension is now owned by fdu-0myw."
---
After the cold and warm loops, validate accepted changes on deterministic 10k/100k/500k corpora plus at least one tens-of-thousands-file real checkout. Record raw evidence hashes, host/build identity, medians and dispersion, profiles, accepted and rejected experiments, complexity rationale, and remaining bottlenecks. Update the plan and PR after every accepted commit and make no product speed claim until the dedicated-host comparator matrix passes.

## Notes

Decision ledger now has 33 validated experiments and final Rust-1.85-compatible exp-032 is the exact cumulative anchor: cold index -54.53%, producer -60.05%, snapshot save -52.41%, warm revalidation -51.99%, snapshot load -35.66% versus b565882 on the 60k real tree. 120k and 720k scale evidence exists for the relevant cold/warm changes. Cold and warm optimization loops are closed; deterministic 10k/100k/500k corpus validation and the dedicated-host comparator matrix remain before any product claim.
