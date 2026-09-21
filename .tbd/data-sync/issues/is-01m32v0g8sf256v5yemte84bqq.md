---
type: is
id: is-01m32v0g8sf256v5yemte84bqq
title: "PR #103 review R3: work-budget test cannot distinguish pricing formulas (Medium)"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6ewht09ryvnt0f5zmhz5
created_at: 2026-09-21T20:37:37.688Z
updated_at: 2026-09-21T21:11:48.763Z
closed_at: 2026-09-21T21:11:48.763Z
close_reason: "Fixed in 5d6e56a2: charge pinned to 6 with the formula, and work.rows_visited asserted equal to the charge."
resolution: null
duplicate_of: null
---
R3 Medium. opened/read.rs:1135-1183 directory_reports_charge_measurement_before_returning_exact_results only asserts charge-1 -> Limit and charge -> Report. Fix: pin the expected value (entries * 2 + shaping) or assert against Work.rows_visited.
