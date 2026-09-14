---
type: is
id: is-01m2ebdjw3vkxk0dppemsjcskw
title: "PR #48 review LIFE-7: the observation handoff fails permanently after three convergent races"
kind: bug
status: open
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:20.482Z
updated_at: 2026-09-14T01:49:45.560Z
---
Low. opened.rs:1236-1272; index.rs:2464-2492. expectation_matches rejects an operation whose baseline moved even when the current state already equals its target, so a refresh committing identical attrs makes the handoff pass stale; each retry is a full-root walk and three in a row end in ObservationHandoffIncomplete -> Failed; retained issues duplicate across retries. Fix: count target-equal mismatches as unchanged; retry only conflicting subtrees. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
