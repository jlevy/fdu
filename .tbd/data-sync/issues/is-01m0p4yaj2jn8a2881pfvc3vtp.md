---
type: is
id: is-01m0p4yaj2jn8a2881pfvc3vtp
title: Probe --no-oracle mode and engine-phase counter scoping
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
created_at: 2026-08-23T01:49:40.801Z
updated_at: 2026-08-23T01:49:40.801Z
---
The headroom review measured the perf_probe oracle digest at ~39% of probe instructions
and 46% of its allocation events, and FDU_COUNTERS tallies include those, so
counter-derived per-entry ratios overstate engine work by a large, job-dependent factor
-- fdu-zgxd was closed as exactly this artifact.

Two cheap fixes, both from the review: a --no-oracle probe mode for attribution runs
(timing runs keep the oracle; an attribution run without it must be labelled as
unverified), and scoping the counter guard to engine phases so harness work is never
counted as engine work. Matters before Phase B of campaign 2, whose profiles will
otherwise attribute oracle cost to the representation being replaced.
