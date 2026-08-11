---
type: is
id: is-01kzs52jemq1q50wy30jdmspqp
title: "session.prioritize(path): verification follows attention"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:34:01.427Z
updated_at: 2026-08-11T19:34:01.427Z
---
The browser knows which directories are on screen; fdu does not. Verification is otherwise breadth-first like the walk, but a prioritised subtree jumps the queue. This is what makes convergence feel immediate rather than merely fast: the handful of rows a user is actually looking at confirm in milliseconds while millions of entries behind them are still unverified. Without it a uniform sweep spends most of its effort on rows nobody is reading.
