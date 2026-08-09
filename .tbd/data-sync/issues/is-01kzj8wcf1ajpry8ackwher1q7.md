---
type: is
id: is-01kzj8wcf1ajpry8ackwher1q7
title: "PR #1 review R1: Reject stale observations and stamp only accepted changes"
kind: bug
status: closed
priority: 0
version: 5
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzj8wcps39f24hgtv04942yw
  - type: blocks
    target: is-01kzj8we2pr4g03f5tdt8n7t08
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:51.968Z
updated_at: 2026-08-09T03:33:09.220Z
closed_at: 2026-08-09T03:33:09.219Z
close_reason: Fixed with Observation versus AppliedDelta separation, per-path baseline preconditions, stale rejection, post-arbitration clocking, and a regression proving delayed reconciliation cannot overwrite newer state.
---
PR #1 review R1. Files: crates/fdu/src/types.rs, crates/fdu/src/index.rs, crates/fdu/src/lib.rs. Separate producer observations from committed changes. Prevent a delayed reconciliation observation from overwriting a newer accepted observation, and mint the public clock only after arbitration. Regression: 10 -> 30 -> delayed 20 must remain 30.
