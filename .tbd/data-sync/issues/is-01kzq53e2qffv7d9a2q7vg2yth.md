---
type: is
id: is-01kzq53e2qffv7d9a2q7vg2yth
title: Scoped revalidation from a changed-dir set
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq53e9225wwa9a978rp2wz3
created_at: 2026-08-11T00:56:00.854Z
updated_at: 2026-08-11T01:09:04.850Z
---
Phase 2b. scan::revalidate_dirs(index, dirs, config, sink): re-list and stat only the named directories, emitting the same conditional Upsert/Remove ops the full sweep would for those directories; MustScanSubDirs becomes InvalidateSubtree resolved by the existing subtree reconcile. CLI gate wiring with --revalidate=auto|full, save-side cursor capture on macOS (captured BEFORE the scan starts so racing mutations replay next open). macOS integration tests: mutate-then-journal-revalidate equals fresh scan by engine digest; UUID mismatch, event-ID regression, forced MustScanSubDirs each degrade to the sweep.

## Notes

Scope extended per operator direction (2026-08-10): must work seamlessly on macOS and fall back seamlessly on Linux, validated end to end in BOTH distribution channels. The journal feature exists on all platforms (G1 fallback compiles everywhere; target-conditional fsevent-sys dependency pulls nothing off-macOS); maturin builds per-platform wheels from the same source so uv/PyPI installs get correct behavior with no extras or markers. CI: ubuntu leg runs the same warm-path e2e through the full-sweep fallback with digest equality; macos leg runs journal integration tests; wheel legs on both OSes exercise a warm open through Python. Acceptance scenario (with fdu-hs10): on the metabrowser-class reference tree, touch exactly one file at depth >= 10, reopen - journal path must equal a fresh scan's engine digest, report exactly the touched directory, and complete in tens of ms with the full sweep as paired control.
