---
type: is
id: is-01m10nsed829d5v3qndj1a2hg7
title: Add explicit optional fdu provider packaging and selection
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nseqmb4w4n271gyre0xnp
parent_id: is-01m0y1sjbfs5h264xhme2vqymg
created_at: 2026-08-27T03:56:31.016Z
updated_at: 2026-08-27T03:56:31.347Z
---
Update inventory_engine/factory.py, pyproject.toml, uv.lock, diagnostics, and startup errors for explicit fdu selection as an optional first-party provider. A missing, incompatible, or unsupported extension fails with typed details and never silently selects Python; a normal install without fdu remains unchanged.
