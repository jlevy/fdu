---
type: is
id: is-01m1dtqxh815zb3zz6m3g11cx6
title: Replace scanner ancestry overlay with a resolved-parent proof
kind: feature
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - design
dependencies:
  - type: blocks
    target: is-01m1dtr3hap1kqbkfcap66paq8
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:33:11.463Z
updated_at: 2026-09-01T06:33:17.609Z
---
Add a private owned ScannerBatch whose preparation proves canonical paths, scope, and each parent as an existing EntryId or earlier batch operation. Consume that numeric proof in detached and opened discovery so they do not build a path-keyed StructuralOverlay, while arbitrary public batches retain atomic preflight.
