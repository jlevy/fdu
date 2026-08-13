---
type: is
id: is-01kzwxffnkwm7kkmrdpgn5rbsn
title: Compare cache-off FDU summary with dumac
kind: task
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
  - dumac
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T06:38:13.682Z
updated_at: 2026-08-13T13:54:54.238Z
closed_at: 2026-08-13T13:54:54.237Z
close_reason: Completed the claim-grade rich-summary versus dumac comparison and matched-workload H64 experiment. The rich result is a statistical wall-time tie with materially lower CPU/RSS; selected-total specialization was rejected and reverted.
---
Run a claim-grade adjacent paired comparison of fdu --cache off --view summary against dumac on the canonical million-scale workspace. Distinguish no persisted snapshot from no retained in-memory index, validate output semantics and hard-link accounting, publish the result, and decide whether H59 should include a true transient summary-only library path.

## Notes

Completed claim-grade cache-off rich-summary comparison with independent files, descendant-directory, apparent-byte, allocated-byte, and newest-file-mtime oracle on every FDU sample. FDU median 3.125 s vs dumac 2.980 s; paired dumac-vs-FDU wall -2.248% CI [-5.733%, +1.689%], statistically unclear. Dumac used +85.447% CPU, +87.795% system CPU, and +224.472% RSS. H64 matched-workload selected-total specialization failed the wall gate and was reverted.
