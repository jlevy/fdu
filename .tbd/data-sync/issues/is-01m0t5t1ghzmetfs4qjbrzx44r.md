---
type: is
id: is-01m0t5t1ghzmetfs4qjbrzx44r
title: "Promotion: per-promoted-tag planes through the reducer path"
kind: feature
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prgyv1eq0g0mzgntn1p4n6
  - type: blocks
    target: is-01m0ptezmtmkn04mh1f1rwgdxb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T15:21:46.769Z
updated_at: 2026-08-24T15:22:35.102Z
---
Promotion: the maintained half of partitioned tallies, priced separately from the tags
it aggregates. A rule may be PROMOTED, and every directory roll-up then maintains one
additional plane for entries carrying that tag, through contribution/merge/unmerge on
the ancestor-merge path -- the path exp-064 took from 43.73% to 14.07% and campaign 2
plans to delete rather than tune. That cost is why promotion is a small declared subset
rather than a property of every tag (decided 2026-08-24: tags are cheap bits, planes
are the cost).

THE SUBTLETY TO GET RIGHT, recorded because the obvious design fails on it:
newest_mtime_ns cannot be derived by subtraction. Files, dirs, bytes, allocated, and
tallies of the complement all derive as all-minus-plane; a maximum does not un-merge.
So the stored plane is the side whose mtime a consumer actually reads -- for gitignore
that is the UNIGNORED side, which is what a browser shows -- and the derived complement
reports newest_mtime as absent rather than wrong. Same principle as ChildRemainder's
deliberately missing mtime field.

WHAT LANDS:
- InternedRollUp per-promoted-tag plane state, as an association list by TagId (the
  by_group precedent: a handful of promoted rules, linear scan beats a map). A plane
  carries the full reducer set: files, dirs, others, bytes, allocated, newest_mtime,
  by_ext, by_group.
- contribution()/merge()/unmerge() extended; the partition property (plane + derived
  complement = all, per promoted tag, across scan, refresh, and watch mutations) as
  property tests in the style the untagged invariants already use.
- Engine reads: rollup/total/tree accept a plane selector; Selection.plane gates the
  precomputed tier exactly as Selection::is_unfiltered does today. ChildSnapshot grows
  per-plane scalar totals so one children() call serves the dual-value row metabrowser
  requires -- scalars only, the per-row map-clone lesson from fdu-plwq applied per
  plane.
- Promotion is declared at open (ScanOptions) and covered by tag_rules_fingerprint, so
  promoting a rule invalidates snapshots exactly like enabling one.

NOT here: --plane, Python Selection(plane=...), and the tagged goldens are fdu-7rwf.
NOT here: pricing -- fdu-n4gn prices this union on a quiet host, and fdu-2ig2's
leaf-count measurement rides the same run.

Blocked by fdu-mvt3. The intended first promoted rule is gitignore, so the surfaces
bead needs both; this bead's own tests promote dotfile to stay dependency-free.
