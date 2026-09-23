---
type: is
id: is-01m35t5cc15exbx9s5x1xkt8fr
title: "PR #105 R2: remove already-used experiment IDs from pickup instructions"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6g0hprgkrq1m6e8rx0f6
created_at: 2026-09-23T00:20:32.256Z
updated_at: 2026-09-23T08:07:03.947Z
closed_at: 2026-09-23T08:07:03.939Z
close_reason: "Fixed in 4d5d5e32 on #105; delta-reviewed"
resolution: null
duplicate_of: null
---
Re-review at 245395c0 confirms prior R2 remains: performance-loop-runbook.md:498-502 offers H149/exp-155, which this PR consumes; later current text says H150/exp-156. Point the entry section at one next-ID statement or update it. No engine/evidence remeasurement required.
