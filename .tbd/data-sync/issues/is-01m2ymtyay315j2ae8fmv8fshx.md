---
type: is
id: is-01m2ymtyay315j2ae8fmv8fshx
title: "H84 screen: Linux adaptive unlock and worker sweep"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels:
  - linux
  - campaign-2
dependencies: []
parent_id: is-01m2ymtwf6fth3a3rk0nn4kw8d
created_at: 2026-09-20T05:32:46.302Z
updated_at: 2026-09-20T06:05:30.389Z
closed_at: 2026-09-20T06:05:30.389Z
close_reason: H84 confirmed silent (~2us/entry, expansions 0). Named-job --threads 8 is not a 3% win (aggregate +1.75% regression). --no-controls is a warm sign, not a shipped PORTABLE constant. fdu-tk1b stays open for bare metal.
---
Measure existing H84/fdu-tk1b on this 4-core VM: confirm adaptive_scale_ups stays 0 and ns/entry stays far below 30us. Screen --threads on aggregate-summary and cold-scan-index. A 3% warm pair is a sign, not a shipped PORTABLE constant. Do not treat as H76 queue-depth evidence.
