---
type: is
id: is-01kzky6vqxwd47xz3we21s86zq
title: Harden fdu against the Rust engineering quality audit
kind: epic
status: open
priority: 1
version: 13
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - engineering-quality
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
child_order_hints:
  - is-01kzky75mq2zkhzvgzs9c95cts
  - is-01kzky7fjvk5f7758cav879nhs
  - is-01kzky7aq9m5j7r8a33tj0tx38
  - is-01kzky7pe77x2wqndf6kdwyn6p
  - is-01kzky7wjz44trprn1ck52pd58
  - is-01kzky86nqp91wq9d3wj2psnwr
  - is-01kzky8bctckj3kk8gwntbg8tn
  - is-01kzky8gxazfdstfgbv3m9fa58
  - is-01kzkyfha9nktqzth06qqf9313
created_at: 2026-08-09T18:57:49.820Z
updated_at: 2026-08-09T20:40:58.173Z
---
Child hardening epic under fdu-qfz6. Fix independent P0 merge blockers fdu-ad45 (executable-dependency trust) and fdu-nlh8 (atomic malformed-batch rejection) first; both feed final approval fdu-sn43. Then pin the toolchain, seal the public API, add model and snapshot fault-state safety nets, and harden stack and native-path boundaries before the Phase 1 representation and publishing work consumes them.
