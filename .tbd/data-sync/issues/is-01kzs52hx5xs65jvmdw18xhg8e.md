---
type: is
id: is-01kzs52hx5xs65jvmdw18xhg8e
title: Provenance transitions on the session stream
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzs52jemq1q50wy30jdmspqp
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:34:00.868Z
updated_at: 2026-08-11T19:34:01.427Z
---
Clearing a UI indicator requires knowing WHEN a value became trustworthy, so the session emits provenance transitions per path alongside value changes. Both outcomes must be reported: verification that CONFIRMS a cached value (clear the mark, no visual jump) and verification that CORRECTS it (update and clear; the UI may want to draw attention). A consumer that only learns about corrections cannot distinguish 'still checking' from 'checked and fine'.
