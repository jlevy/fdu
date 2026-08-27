---
type: is
id: is-01m10nq3svzaygbyh3mvmkt0g7
title: Implement the disposable unchanged-contract fdu adapter
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nq43t0s22axb5bsbhnnfy
parent_id: is-01m0y1sfedr9qf3sc7e4bf6fd7
created_at: 2026-08-27T03:55:14.618Z
updated_at: 2026-08-27T06:12:05.778Z
closed_at: 2026-08-27T06:12:05.777Z
close_reason: Completed on MetaBrowser branch codex/fdu-opened-root-e2e-spike at commit 2743064 against exact fdu wheel revision 0583a1a. The normalized evidence and reproduction commands are under explorations/fdu-inventory-adapter; MetaBrowser make verify and strict exact-wheel typing pass. The disposable adapter exercises the unchanged five-operation contract without shipping registration or fallback.
resolution: null
duplicate_of: null
---
Under MetaBrowser explorations/fdu-inventory-adapter, map PyOpenedIndex to the existing InventoryHandle protocol and all eight query forms. Naive full materialization, sorting, remainder counting, and aggregate scans are permitted only here, must be visibly counted, and must not enter the shipping factory or retain a second uninstrumented authority.
