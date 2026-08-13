---
type: is
id: is-01kzx08n0ntz7ckxyv8q4msv23
title: "H62: Reduce transient summaries inside scan workers"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzwk20kyaxajq254tee8apts
created_at: 2026-08-13T07:26:55.509Z
updated_at: 2026-08-13T10:08:31.978Z
closed_at: 2026-08-13T10:08:31.978Z
close_reason: Measured and rejected as a standalone production change in exp-041; only a later combined H62+H63 verdict may revive the mechanism.
---
For the derived uncached unfiltered-summary plan, aggregate exact files/directories/apparent bytes/allocated bytes/newest file mtime inside the existing fixed scan workers. Avoid file PathBuf joins, Op/Observation batches, and the single consumer channel while preserving scope, errors, portability, and byte-identical Report output. Profile first; accept only with >=3% paired wall improvement and exact semantic hashes on the million-entry tree.

## Notes

Exp-041 rejects H62 standalone. Exact worker-local prototype on mutation-free 901,963 entries: wall -1.377% [-3.705%, -0.306%], below 3% bar, despite user CPU -36.23%, RSS -34.77%, faults -28.44%, involuntary context switches -10.92%, system CPU unchanged. Independent 720,805-entry 20-pair run: wall -1.264% [-3.101%, +0.585%], same mechanism. Zero invalid samples, semantic mismatches, drift, or mutation. Prototype remains temporarily only to screen preregistered H63 composition; it will be reverted if combined wall still misses.
