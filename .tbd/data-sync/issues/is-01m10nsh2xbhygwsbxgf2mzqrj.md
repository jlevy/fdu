---
type: is
id: is-01m10nsh2xbhygwsbxgf2mzqrj
title: Add exact-revision cross-repository wheel CI
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0y1sk24z37hnvpxee6apg8e
created_at: 2026-08-27T03:56:33.756Z
updated_at: 2026-08-27T03:56:33.756Z
---
Build fdu from the pinned PR #48 revision, install the produced wheel into a clean MetaBrowser job, verify no sibling checkout or source-tree import is reachable, and run the provider registry plus installed lifecycle across supported Python/platform jobs. Record both exact revisions in job artifacts and PR validation.
