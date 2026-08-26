---
type: is
id: is-01m0t5t2sa2rn3qm3m4dycv7hv
title: Fold Classification.flags into the tag model as Name-tier rules
kind: task
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T15:21:48.073Z
updated_at: 2026-08-26T07:01:50.826Z
closed_at: 2026-08-25T06:23:26.410Z
close_reason: |
  Two of the three folded in; the third cannot, and the tag model's own tier check is the
  reason rather than an oversight.

  `vendored` and `documentation` ship as catalogue rules at `TagTier::Path` -- they read the
  relative path rather than just a basename, which is what that tier means. The bead said
  Name-tier; that was wrong about both of them.

  They needed a new matcher shape. Path-tier previously meant `gitignore`, which needs state
  gathered from the tree, so `needs_path` and `needs_binding` had collapsed into one
  question. A pure-path rule reads the path and nothing else: decided the moment an entry
  lands, nothing to bind. `Matcher::PurePath` splits those, and `needs_path` is no longer
  gated on the gitignore feature.

  The classification reports both unchanged -- a consumer reading `flags.vendored` needs no
  tag rule enabled -- and now calls the same predicate the tag does. That is the whole point:
  a caller filtering with `--not-tag vendored` and a row saying `vendored: true` cannot
  disagree about a file, because there is no second copy of the rule to drift.

  `generated` stays a classification flag alone. It reads the file's opening bytes, which is
  `TagTier::Content`, and `TagRules::from_names` refuses that tier rather than turning a
  metadata walk into a content walk -- the exact failure the tier check exists for, whose only
  symptom would be that scans got mysteriously slower. It is available on the classification
  of a file whose bytes were read for some other reason, which is the only place it is free.

  The fold found the drift it was meant to prevent. The two copies of the stem check had
  already diverged: the classification's used `get(..len)` and the newer one indexed, and the
  indexing form panics on a name whose stem length lands inside a multi-byte character. One
  function now, and the test names `réadme.md` for that reason.

  Goldens: the catalogue listing in two "unknown tag rule" messages, which now names four
  rules. Parity holds with no artifact change.
resolution: null
duplicate_of: null
---
Classification.flags (generated, vendored, documentation) are per-entry booleans
recomputed from the name on every query and never maintained -- Name-tier tag rules
wearing a different hat, found during the 2026-08-24 genericity review. Fold them into
the tag model: each becomes an available Name-tier rule, classification keeps reporting
them unchanged for compatibility, and a consumer wanting "bytes excluding vendored"
gets it by tag instead of by a bespoke walk.

Not before the model settles: this moves goldens and is pure consolidation, so it waits
for fdu-mvt3 and rides behind the planes work. P3.
