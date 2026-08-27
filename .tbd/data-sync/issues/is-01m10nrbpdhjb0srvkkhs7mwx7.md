---
type: is
id: is-01m10nrbpdhjb0srvkkhs7mwx7
title: Update MetaBrowser runtime and routes for the revised provider contract
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0y1sgqd1sd33stssgw25f2q
created_at: 2026-08-27T03:55:55.468Z
updated_at: 2026-08-27T03:55:55.468Z
---
Update runtime.py default_inventory_config, inventory_provider_from_environment, InventoryRuntime open/replace_root/close, and server inventory routes. Pass actual registry content and explicit scope, render capped totals honestly, preserve public envelopes, and keep provider choice and failures explicit with no filesystem or aggregation fallback.
