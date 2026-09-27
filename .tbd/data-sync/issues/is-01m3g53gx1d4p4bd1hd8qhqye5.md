---
type: is
id: is-01m3g53gx1d4p4bd1hd8qhqye5
title: Use shared Unix cache location and descriptive metadata and analysis filenames
kind: feature
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
delegate: sol-cache-layout
labels: []
dependencies:
  - type: blocks
    target: is-01m3g53hk9nr8ff52d3xj77ttj
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
hold: null
hold_until: null
created_at: 2026-09-27T00:44:10.016Z
updated_at: 2026-09-27T03:12:09.159Z
started_at: 2026-09-27T01:19:37.676Z
closed_at: 2026-09-27T03:12:09.159Z
close_reason: Implemented across engine, CLI, Python, schemas and docs in PR133.183 shared CLI goldens,886 core tests,68 cross-format cases, full16,787-case path matrix, cache fault proofs, and Apple/Windows cross-lint pass. Final packaging/CI handoff remains tracked by fdu-7jtp.
resolution: null
duplicate_of: null
---
Implement the approved macOS ~/.cache/fdu default and shared configurable destination (typed core/CLI cache-dir, FDU_CACHE_DIR, XDG, platform defaults). Rename conventional sibling files to <root-key>.metadata.bin and <root-key>.analysis.bin, with one core path-pair model covering save/load/status/clear/staging/orphans. Document current file roles and precedence. Preserve identity checks, magic recognition, atomic writes, symlink/unknown-file safeguards and active-staging protection. User explicitly excludes old-location docs, legacy discovery, migration and compatibility work. Automatic retention remains fdu-558j.
