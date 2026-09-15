---
type: is
id: is-01m2gbkwthfddzsg1ct5ggzkbn
title: "Handoff convergence rule misses control ops: a .gitignore edit racing the handoff still goes stale"
kind: bug
status: closed
priority: 3
version: 8
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h3ches22p7k9x4kka6bq6q
created_at: 2026-09-14T16:22:16.135Z
updated_at: 2026-09-15T00:29:45.272Z
closed_at: 2026-09-15T00:29:45.268Z
close_reason: "705afe3: Index::holds_target extends target equality to ControlUpsert (exact source retained) and ControlRemove (none retained); handoff race test now covers a .gitignore, plus a unit test for converging and diverging control ops."
resolution: null
duplicate_of: null
---
PR #56 review PR56-LIFE-1 (https://github.com/jlevy/fdu/pull/56#pullrequestreview-5200187448). Locations at cfd1335: index.rs:4855-4860 and :3272-3276, scan.rs:4291-4304. The LIFE-7 fix (fdu-dkr0) applies an op as unchanged when the index already holds its target, but only for Upsert/Remove. An Upsert plus ControlUpsert pair on one baseline still goes stale when a refresh converged first. A .gitignore edit racing the handoff therefore still costs a full-root walk, and three such races in a row still fail the root. Fix: extend the target-equality check to ControlUpsert/ControlRemove (the control source fingerprint already equals the target), and add a gate-driven race test.
