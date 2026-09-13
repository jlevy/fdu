---
type: is
id: is-01m2eag5hr11xx52vv30jzswaa
title: "PR #51 review COMMIT-2: the watch event path ignores read_controls"
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - control-state
  - correctness
dependencies: []
parent_id: is-01m2eafpfpe8k5c9z9dhrqvy2y
created_at: 2026-09-13T21:24:16.567Z
updated_at: 2026-09-13T21:24:16.567Z
---
PR #51 review COMMIT-2 (High), https://github.com/jlevy/fdu/pull/51#pullrequestreview-5192254822.

crates/fdu-core/src/watch.rs:901, 913 (verify_intent) and 599, 607 (reverify_observation) call scan::read_control_op_unconditional, so a watch started with read_controls: false still reads .gitignore files touched by events and emits ControlUpsert. The index then classifies from a partial rule set under a scope whose ignore_rules_fingerprint is 0, and the snapshot carries control sources under that fingerprint.

Fix: route all four sites through the gated scan::read_control_op(scan_config, ...), keep the unconditional form only for the one test that needs it (scan.rs control-read test), adopt the review's review_verify_intent_ignores_read_controls_opt_out proof test as a regression test.
