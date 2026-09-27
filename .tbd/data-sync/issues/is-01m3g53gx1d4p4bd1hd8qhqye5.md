---
type: is
id: is-01m3g53gx1d4p4bd1hd8qhqye5
title: Use shared Unix cache location and descriptive metadata and analysis filenames
kind: feature
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m3g53hk9nr8ff52d3xj77ttj
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
created_at: 2026-09-27T00:44:10.016Z
updated_at: 2026-09-27T00:50:04.636Z
---
Implement the approved macOS ~/.cache/fdu default and shared configurable destination (typed core/CLI cache-dir, FDU_CACHE_DIR, XDG, platform defaults). Rename conventional sibling files to <root-key>.metadata.bin and <root-key>.analysis.bin, with one core path-pair model covering save/load/status/clear/staging/orphans. Document current file roles and precedence. Preserve identity checks, magic recognition, atomic writes, symlink/unknown-file safeguards and active-staging protection. User explicitly excludes old-location docs, legacy discovery, migration and compatibility work. Automatic retention remains fdu-558j.
