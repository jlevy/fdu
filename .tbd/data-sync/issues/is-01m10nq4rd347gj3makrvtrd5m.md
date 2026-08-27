---
type: is
id: is-01m10nq4rd347gj3makrvtrd5m
title: Publish spike evidence and quarantine the disposable adapter
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0y1sfedr9qf3sc7e4bf6fd7
created_at: 2026-08-27T03:55:15.596Z
updated_at: 2026-08-27T06:12:06.477Z
closed_at: 2026-08-27T06:12:06.476Z
close_reason: Completed on MetaBrowser branch codex/fdu-opened-root-e2e-spike at commit 2743064 against exact fdu wheel revision 0583a1a. The normalized evidence and reproduction commands are under explorations/fdu-inventory-adapter; MetaBrowser make verify and strict exact-wheel typing pass. The experiment code is quarantined, not packaged, registered, selectable, or a production dependency; Checkpoint 3C owns replacement and deletion.
resolution: null
duplicate_of: null
---
Publish normalized exact-revision evidence and a reproducible harness. Keep the disposable adapter isolated under explorations, excluded from packaging and provider selection, until Checkpoint 3C replaces it; the final thin-adapter gate deletes it. Close only after repository gates and exact-wheel typing pass.
