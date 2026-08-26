---
type: is
id: is-01m0y1scsff119ypyb93tbbnxh
title: Harden admission, detached images, features, and identity
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sd4tmt95d9tdsynn7v6g
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:28.974Z
updated_at: 2026-08-26T11:33:43.891Z
closed_at: 2026-08-26T11:33:43.882Z
close_reason: Centralized filesystem admission across cold scan, reconcile, refresh, and watch; made detached image and semantic identities explicit; restored the empty core feature floor; recorded dependency and binary baselines; and passed focused tests, make check, and make cross-lint.
resolution: null
duplicate_of: null
---
Complete Checkpoints 1C and 1D: fixed hidden/symlink/filesystem/object-kind admission, platform parity including macOS bulk and Windows paths, detached Index image versus live authority, snapshot migration, derived scope and semantic identity, empty default features, dependency audit, and binary/wheel size baselines.
