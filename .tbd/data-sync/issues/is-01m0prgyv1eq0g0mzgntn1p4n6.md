---
type: is
id: is-01m0prgyv1eq0g0mzgntn1p4n6
title: "Partitioned tallies surfaces: --tags and --plane, Selection.plane, per-plane values"
kind: feature
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:54.336Z
updated_at: 2026-08-25T05:05:39.919Z
closed_at: 2026-08-25T05:05:39.918Z
close_reason: |
  Shipped across all three surfaces.

  Command line: `--promote LIST` on Scope and `--plane TAG` on Selection, kept apart
  because promotion moves the snapshot fingerprint and a Selection flag that invalidated
  a cache would repeat the `--not-tag` mistake the tag model already rejected. Three flags
  to reach one number is the honest price of a cache-correct model. Axis table row, and
  `--plane` errors name the step that would fix them: promote the rule, enable it, or
  correct the spelling -- three distinct answers rather than one "unknown".

  Python: `ScanOptions(promote=)`, `Selection(plane=)`, `Child.plane` beside
  `Child.totals` so one `children(plane=...)` call serves the dual-value listing, and
  `plane=` on `total()`, `rollup()` and `children()`.

  Engine: `TagRules::plane_of` and `promoted_names`, `TagRuleError::NotPromoted`,
  `Index::plane_rollup_bounded`/`plane_rollup_of`/`plane_scalars_of`,
  `ChildPageRequest.plane`, `ChildSnapshot.plane_totals`, `Selection.plane`, and the
  section builders routed through `maintained_scalars`/`maintained_total`.
  `Selection::plane` stays outside `is_unfiltered` so a plane query is a roll-up read, and
  acts as one more exclusion on the walking tier so the two agree.

  Goldens: seven new sessions on the existing tagged `repo` fixture in
  cli-axes.tryscript.md, replayed by the parity harness as seven new exact matches -- no
  new declared deviation. Default behaviour is byte-identical: no plane named, nothing
  moves.

  Exposing the state found three defects in it, all of the same shape -- a plane read is
  fast because it reads state maintained elsewhere, so a wrong plane is indistinguishable
  from a right one by inspection:

  1. `ensure_dir_chain` built its placeholder directory's contribution by hand as
     `dirs: 1` with no planes, and on a real walk nearly every directory is materialised as
     an ancestor before it is observed. A plane's directory count was near zero while its
     files and bytes were exactly right. Fixed with one shared `count_dir_into_planes`.
  2. A rebind re-tagged every entry and left the planes derived from the old bits. Since
     a Path-tier rule's bits are only ever correct after that rebind, `gitignore` -- the
     rule planes exist for -- reported a plane equal to the whole tree. `rebuild_planes`
     now runs when any bit moved.
  3. An unfiltered `--view summary` was answered by the tier that retains aggregate
     tallies and no index, which holds no plane. It did not fail when asked for one; it
     returned the whole tree under the plane's heading.

  None is visible from one tier. `crates/fdu-core/tests/plane_equivalence.rs` scans a real
  tree and requires the maintained plane and a walk over the same restriction to agree, at
  every directory, for a Name-tier and a Path-tier rule, with and without a rebind. Each
  of the three fails it when reverted.

  Also fixed a silent gap in the Python smoke harness: it calls each check by name from
  `main`, so a check that is written and never listed passes forever. It now parses its
  own source and asserts every `check_` it defines is called.
resolution: null
duplicate_of: null
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
