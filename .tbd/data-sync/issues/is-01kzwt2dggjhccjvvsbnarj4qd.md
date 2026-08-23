---
type: is
id: is-01kzwt2dggjhccjvvsbnarj4qd
title: Compare diskus benchmark protocol with FDU evidence protocol
kind: task
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - performance
  - research
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T05:38:39.759Z
updated_at: 2026-08-23T02:11:33.033Z
closed_at: 2026-08-13T05:51:09.408Z
close_reason: Reviewed current diskus upstream README and repository. Captured its Linux per-sample drop_caches, separate warm regime, and parameter-scan practices; synchronized the Linux plan while retaining FDU's stronger oracle/provenance protocol.
---
Review current diskus README and benchmark assets, document its corpus/cache/timing approach, compare it with FDU's paired oracle-validated protocol, and adopt any stronger practice without weakening semantic work-class labeling or provenance.

## Notes

Current upstream master 90196e9 retains the v0.9.0 README protocol: Hyperfine on 100k dirs/400k files; Linux controlled-cold cache via sync + drop_caches before each run; warm cache via five warmups; Hyperfine parameter-scan for tin-summer thread count. The repository has no separate benchmark harness/assets beyond the README. Adopt explicit per-sample controlled-cold Linux preparation and retain verified-warm separately. FDU adds paired adjacent scheduling, exact binary/source/host/tree provenance, mutation checks, correctness oracle, work classes, hard-link prevalence, resource metrics, stable-output checks, and bootstrap confidence intervals.
