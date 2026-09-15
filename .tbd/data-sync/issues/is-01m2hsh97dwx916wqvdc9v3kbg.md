---
type: is
id: is-01m2hsh97dwx916wqvdc9v3kbg
title: "PR #55 review PR55-CLASS-1: comparability has no state for partially observed ignore classification"
kind: bug
status: in_progress
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2hsgt9d35edzxt2rqvfc3pv
created_at: 2026-09-15T05:44:45.035Z
updated_at: 2026-09-15T05:44:50.458Z
---
PR #55, delta review 5205945198. Plan @55ce4a3 :409-426 decides classification comparability from SemanticIdentity alone, with one degraded state (captured without control observation); :227-234 typed gaps cannot carry it because sizes are exact. fdu-1onj records the user's PR A decisions for 0.1.0: a crossed control budget is an exit-0 coverage note with exact sizes (Q1), the budget is mixed into ignore_rules_fingerprint (Q2), the per-line guard is liftable by the same flag (Q3), and snapshot FORMAT_VERSION is bumped to carry refused rules (Q11). Two checkpoints with equal SemanticIdentity but different refused-rule sets would compare ignored/unignored partitions as if both were complete. Fix: a checkpoint records its refused rule sources and control budget; classification deltas across refusals are marked partial for the affected directories (or refused; pick one with the case against); size deltas stay exact; cross-reference fdu-1onj and PR A; add a correctness case; note on fdu-8ybz.
