---
type: is
id: is-01kzq53e2qffv7d9a2q7vg2yth
title: Scoped revalidation from a changed-dir set
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq53e9225wwa9a978rp2wz3
created_at: 2026-08-11T00:56:00.854Z
updated_at: 2026-08-11T00:56:07.590Z
---
Phase 2b. scan::revalidate_dirs(index, dirs, config, sink): re-list and stat only the named directories, emitting the same conditional Upsert/Remove ops the full sweep would for those directories; MustScanSubDirs becomes InvalidateSubtree resolved by the existing subtree reconcile. CLI gate wiring with --revalidate=auto|full, save-side cursor capture on macOS (captured BEFORE the scan starts so racing mutations replay next open). macOS integration tests: mutate-then-journal-revalidate equals fresh scan by engine digest; UUID mismatch, event-ID regression, forced MustScanSubDirs each degrade to the sweep.
