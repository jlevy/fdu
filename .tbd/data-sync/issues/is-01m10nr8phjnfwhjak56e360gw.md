---
type: is
id: is-01m10nr8phjnfwhjak56e360gw
title: Revise MetaBrowser inventory configuration and derived identity
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nr925xb4ybt40q2pw7zpn
parent_id: is-01m0y1sfw7kwjprd6sfky281fj
created_at: 2026-08-27T03:55:52.400Z
updated_at: 2026-08-27T06:13:49.824Z
---
Update inventory_engine/contract.py InventoryConfig, validation, and inventory_scope_fingerprint: pass immutable registry content, introduce DiscoveryBudget execution policy, move depth to query selection, name hidden/symlink/filesystem/object-kind scope, derive identities from validated values, version the encoding, and intentionally invalidate prototype caches without compatibility shims.
