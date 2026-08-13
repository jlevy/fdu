---
type: is
id: is-01kzwt2d8dapafh6tmf9f92gek
title: Extend comparative evidence to Linux
kind: task
status: open
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - linux
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T05:38:39.500Z
updated_at: 2026-08-13T14:34:04.038Z
---
Repeat the claim-grade paired comparison on a controlled Linux local-SSD host using the portable backend, then add Linux numbers to the comparison report and white paper. Include exact binary/host/corpus provenance, the full oracle, pre/post fingerprint, a controlled or explicitly classified OS-cache state, and profile-backed follow-up hypotheses such as statx/getdents/io_uring only when measurements justify them.

## Notes

Queued after the M1/APFS report. Publish two separate matrices on a controlled Linux local-SSD host: controlled-cold with successful sync plus per-sample drop_caches preparation, and verified/repeated-workload warm with explicit full-tree warmups. Keep exact-binary, paired adjacency, pre/post fingerprint, semantic work-class, oracle, resource, and bootstrap-CI requirements. Do not extrapolate warm ranking or effect size into the cold regime; diskus's published 10.18x cold versus 2.20x warm gap illustrates why both must be measured.
