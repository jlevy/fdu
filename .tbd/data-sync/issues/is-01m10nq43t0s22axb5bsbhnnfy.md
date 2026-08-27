---
type: is
id: is-01m10nq43t0s22axb5bsbhnnfy
title: Instrument provider and route costs on one shared corpus
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nq4dv23nfwsghy63h8628
parent_id: is-01m0y1sfedr9qf3sc7e4bf6fd7
created_at: 2026-08-27T03:55:14.937Z
updated_at: 2026-08-27T03:55:15.258Z
---
Measure both the disposable fdu adapter and PythonInventoryBackend through the same InventoryHandle requests and route calls. Record rows visited/returned, sorts, materialized bytes, aggregate passes, binding bytes, latency, peak memory, visible order, visible totals, page behavior, and time to first useful response without changing the contract under test.
