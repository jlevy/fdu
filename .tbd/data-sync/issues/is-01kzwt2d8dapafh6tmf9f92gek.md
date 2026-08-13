---
type: is
id: is-01kzwt2d8dapafh6tmf9f92gek
title: Extend comparative evidence to Linux
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - linux
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T05:38:39.500Z
updated_at: 2026-08-13T05:47:50.805Z
---
Repeat the claim-grade paired comparison on a controlled Linux local-SSD host using the portable backend, then add Linux numbers to the comparison report and white paper. Include exact binary/host/corpus provenance, the full oracle, pre/post fingerprint, a controlled or explicitly classified OS-cache state, and profile-backed follow-up hypotheses such as statx/getdents/io_uring only when measurements justify them.

## Notes

Queued after the M1/APFS report. Incorporate diskus's per-sample Linux sync/drop_caches preparation as a controlled-cold state, retain a separate verified-warm regime, and use FDU's exact-binary, paired, pre/post fingerprint, semantic work-class, oracle, resource, and bootstrap-CI requirements.
