---
type: is
id: is-01kzvqqawcnm795f25dd6dpbbg
title: "Revalidate PR #8 performance against post-PR-#5 main"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-12T19:38:25.035Z
updated_at: 2026-08-23T02:11:33.033Z
closed_at: 2026-08-13T05:51:07.884Z
close_reason: Revalidated against merged post-PR-5 main at 60k, 720k, and million scale with exact-oracle paired experiments 033-035; the performance stack remains a strict speed improvement.
---
Build post-#5 origin/main and the merged PR #8 candidate from equivalent release configurations; re-profile the live cold and warm paths; run interleaved paired exact-oracle comparisons at decision-grade trial counts; include current dust calibration where available; and record whether the cumulative speedup survives the new baseline.
