---
type: is
id: is-01kzx08n0ntz7ckxyv8q4msv23
title: "H62: Reduce transient summaries inside scan workers"
kind: task
status: in_progress
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzwk20kyaxajq254tee8apts
created_at: 2026-08-13T07:26:55.509Z
updated_at: 2026-08-13T09:49:42.084Z
---
For the derived uncached unfiltered-summary plan, aggregate exact files/directories/apparent bytes/allocated bytes/newest file mtime inside the existing fixed scan workers. Avoid file PathBuf joins, Op/Observation batches, and the single consumer channel while preserving scope, errors, portability, and byte-identical Report output. Profile first; accept only with >=3% paired wall improvement and exact semantic hashes on the million-entry tree.

## Notes

Starting from the committed H59 exact-summary baseline. Native profilers are currently unavailable (/usr/bin/sample empty; xctrace aborts in Xcode Devices plugin), so pre-change attribution is the exact-binary resource signature: index removal cuts user CPU ~66% while system CPU is unchanged and dominates uniform trees. H62 targets remaining generic file PathBuf/Op/channel work inside existing workers, with strict exact-output and >=3% paired-wall gate.
