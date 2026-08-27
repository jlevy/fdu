---
type: is
id: is-01m10nr9f2mkwdtp8ad88ms621
title: Close the revised MetaBrowser provider conformance registry
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0y1sfw7kwjprd6sfky281fj
created_at: 2026-08-27T03:55:53.184Z
updated_at: 2026-08-27T03:55:53.184Z
---
Update tests/test_inventory_provider_contract.py and the architecture registration table so every configuration field, enum value, query/result variant, path rule, count bound, page outcome, change/reset, refresh, cancellation, and close outcome has a provider-independent case. Make registration/exhaustiveness tests fail on an unmapped value before either provider implementation changes.
