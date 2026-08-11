---
type: is
id: is-01kzpvt29hvtsrg1pyrq20awxa
title: Publish the real-tree optimization decision ledger
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-10T22:13:36.688Z
updated_at: 2026-08-11T00:24:48.121Z
---
After the cold and warm loops, validate accepted changes on deterministic 10k/100k/500k corpora plus at least one tens-of-thousands-file real checkout. Record raw evidence hashes, host/build identity, medians and dispersion, profiles, accepted and rejected experiments, complexity rationale, and remaining bottlenecks. Update the plan and PR after every accepted commit and make no product speed claim until the dedicated-host comparator matrix passes.

## Notes

Partly delivered. The decision ledger exists and is generated: docs/project/reports/report-2026-08-10-fdu-performance-experiments.md, built by 'make perf-ledger' from validated soft-schema artifacts in docs/project/experiments/. It records accepted and rejected experiments with complexity rationale, host and tree identity (content digest, CPU, cores, memory, filesystem, toolchain, binary hashes), medians and 95% bootstrap intervals, and the remaining bottlenecks. Still outstanding for closure: multi-scale validation on the deterministic 10k/100k/500k corpora, and no product speed claim until the dedicated-host comparator matrix passes.
