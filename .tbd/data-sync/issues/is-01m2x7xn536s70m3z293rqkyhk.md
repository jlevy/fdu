---
type: is
id: is-01m2x7xn536s70m3z293rqkyhk
title: "H121: post-H115+H120 cache-hit restore re-profile"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
created_at: 2026-09-19T16:27:49.282Z
updated_at: 2026-09-19T16:27:49.282Z
---
After H115 (restore rebuild) and H120 (streaming parse-into-apply), the exp-109 restore mix is stale. Profile first. A named apply/install cut follows only if apply still dominates (that leftover is H83). Not another alloc trim. Not a retry of H116.

Metric: deciding-scale content-cache-hit stage split on metabrowser-clone (timers already in). Determination: a named restore stage is at least 50% of restore phase time and at least 3% of claim-grade wall. Counters off for the wall pair. Digest identical.

What refutes: apply no longer dominates (scope H83 down), or no stage clears the bar.

Why significant: two accepted restore changes landed after the mix was measured. Do not start an apply cut from exp-109.

Protocol: docs/project/guides/performance-loop.md. Pickup: runbook Current Standing. Parent epic: fdu-8ya1.
