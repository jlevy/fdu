---
type: is
id: is-01m10nsgdmz0ns8rmgq0p5mfkm
title: Prove one complete installed-wheel browser lifecycle with fdu
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nsgrhs1bz3js9cqz29g85
parent_id: is-01m0y1sk24z37hnvpxee6apg8e
created_at: 2026-08-27T03:56:33.075Z
updated_at: 2026-08-27T03:56:33.424Z
---
Extend browser lifespan and filesystem-to-SSE tests with one causal lifecycle: explicit fdu selection, cold open, useful progressive read, completion, real mutation, change relay, coherent reread, refresh, root replacement, iterator cancellation, and joined close. Use the built exact-revision wheel in an isolated environment.
