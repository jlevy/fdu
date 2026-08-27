---
type: is
id: is-01m10nr8phjnfwhjak56e360gw
title: Revise MetaBrowser inventory configuration and derived identity
kind: task
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nr925xb4ybt40q2pw7zpn
parent_id: is-01m0y1sfw7kwjprd6sfky281fj
created_at: 2026-08-27T03:55:52.400Z
updated_at: 2026-08-27T06:53:51.031Z
closed_at: 2026-08-27T06:53:51.029Z
close_reason: Joint scope, execution-policy, selection, and registry-identity contract implemented and fully validated at MetaBrowser 0a6ddbb.
resolution: null
duplicate_of: null
---
Update inventory_engine/contract.py InventoryConfig, validation, and inventory_scope_fingerprint: pass immutable registry content, introduce DiscoveryBudget execution policy, move depth to query selection, name hidden/symlink/filesystem/object-kind scope, derive identities from validated values, version the encoding, and intentionally invalidate prototype caches without compatibility shims.

## Notes

Completed in MetaBrowser commit 0a6ddbb on codex/fdu-opened-root-e2e-spike. InventoryConfig now accepts immutable registry content and DiscoveryBudget, names the fixed v1 hidden/symlink/filesystem/object-kind scope, excludes execution budget and query depth from the versioned scope identity, and rejects unsupported values. The Python provider derives semantic identity from the parsed document and uses the same registry for filtering, navigation tallies, and rollups. Full MetaBrowser verification: 1,623 pytest cases, 48 CLI goldens, lint/type/audit/distribution checks.
