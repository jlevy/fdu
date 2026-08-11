---
type: is
id: is-01kzs52haatwf30skqvs3vd3p1
title: "Session: start a scan, read growing results, cancel"
kind: task
status: open
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzs52hx5xs65jvmdw18xhg8e
  - type: blocks
    target: is-01kzs52k0k897fpdjngr9yhhh4
  - type: blocks
    target: is-01kzs52ksq9znnf5p0pcan8rpc
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:34:00.265Z
updated_at: 2026-08-11T19:34:02.806Z
---
Without this, breadth-first ordering has no consumer and partial results are unreachable from outside - it is what makes the landed order change useful. Session::start(root, config) returns immediately; report(&query) reads roll-ups plus per-path provenance at any time; is_complete(); cancel() stops promptly. Builds on IndexHandle, which already serves readers while a producer applies short writes. Documented and tested contract: under BreadthFirst every roll-up a consumer observes is a lower bound that only increases until its subtree completes. Property test samples during a scan and asserts no observed total ever decreases. Bounded-memory option is a session concern, not an engine one. Supersedes the streaming half of fdu-avle.
