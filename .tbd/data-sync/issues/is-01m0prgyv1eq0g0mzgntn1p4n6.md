---
type: is
id: is-01m0prgyv1eq0g0mzgntn1p4n6
title: "Partitioned tallies surfaces: --tags and --plane, Selection.plane, per-plane values"
kind: feature
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:54.336Z
updated_at: 2026-08-24T15:23:03.069Z
---
Surfaces for the tag model and its planes, re-scoped 2026-08-24 when tags and planes
were decoupled. --tags (scope: enable rules) and the tag filter land with the
foundation bead fdu-mvt3; this bead completes the rest once promotion exists:

- CLI: --plane <rule> on the selection axis, taking a PROMOTED rule's name from the
  declared set -- no longer one-plane-per-enabled-tag. Axis table row; an unpromoted or
  unknown name errors listing the promoted set, in the house style.
- Python: Selection(plane=...), promotion on ScanOptions, per-plane values on RollUp
  and Child rows (the dual-value listing in one children() call), tag names on rows.
- Default plane `all` keeps untagged behaviour byte-identical: the golden corpus must
  not move for anyone not using tags.
- Goldens with a tagged fixture in every format, replayed by the parity harness. The
  tagged fixture is also what fdu-ey9q's progressive goldens need.

Blocked by fdu-pxfz (promotion) and fdu-brt0 (the fixture worth golden-testing is a
gitignored tree).

## Notes

Re-scoped 2026-08-24; the SHAPE UNDER REVIEW question is decided and applied — see fdu-mvt3's description and the Phase 1 rewrite in the integration spec. --plane takes a promoted rule's name from a declared set.
