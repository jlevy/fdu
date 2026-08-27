---
type: is
id: is-01m10nsdjx7z9h87m4nf8hzhyh
title: Implement exhaustive fdu-to-MetaBrowser value and query translation
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nse28b6m4gajwbks6a08g
parent_id: is-01m0y1sjbfs5h264xhme2vqymg
created_at: 2026-08-27T03:56:30.171Z
updated_at: 2026-08-27T03:56:30.663Z
---
Add providers/fdu_inventory.py FduInventoryBackend and its private handle. Map configuration, canonical paths, all eight queries/results, lifecycle/coverage/freshness/source, work, issues, impact domains, and continuations exhaustively; retain only the native handle and conversion code, with no walker, row replica, aggregate store, sort, fallback, or private identity recipe.
