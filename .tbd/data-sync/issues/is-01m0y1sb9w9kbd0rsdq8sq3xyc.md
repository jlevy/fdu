---
type: is
id: is-01m0y1sb9w9kbd0rsdq8sq3xyc
title: "Land the audited PR #47 runtime TypeRegistry commit"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sbmnpxmcyt3rvm7qt1rg
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:27.451Z
updated_at: 2026-08-26T03:28:54.118Z
---
Immediately after 1A and before broad index.rs edits, apply PR #47 commit 9b31220 with cherry-pick --no-commit. Audit all hunks, retain the shared manifest parser, runtime TypeRegistry, registry-owned classification, derived fingerprints, and migration tests, omit later surface flags, and gate default/no-default, snapshot, content, and classification behavior.
