---
type: is
id: is-01kzkw4qwdk1350g9y4y9fgjkd
title: Validate scan batching and unsupported filesystem boundaries before allocation
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzkw4ddv9g9jry50tp4xzgtw
created_at: 2026-08-09T18:21:43.180Z
updated_at: 2026-08-09T18:31:47.252Z
closed_at: 2026-08-09T18:31:47.252Z
close_reason: Implemented with regression coverage; the complete local handoff gate passes.
---
ScanConfig accepts batch_size zero and arbitrary usize values; cold scan and reconcile interpret zero differently and with_capacity can attempt an attacker- or caller-sized allocation. Define and validate a bounded nonzero range before allocating. On platforms without device identity, one_filesystem must fail explicitly rather than silently traverse every device.
