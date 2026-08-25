---
type: is
id: is-01m0t5t1ghzmetfs4qjbrzx44r
title: "Promotion: per-promoted-tag planes through the reducer path"
kind: feature
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prgyv1eq0g0mzgntn1p4n6
  - type: blocks
    target: is-01m0ptezmtmkn04mh1f1rwgdxb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T15:21:46.769Z
updated_at: 2026-08-25T04:15:42.347Z
closed_at: 2026-08-25T04:15:42.346Z
close_reason: |
  Shipped, engine-internal. `make check` green, parity holds. The public surfaces -- `--plane`,
  `Selection.plane`, per-plane values on `RollUp`/`Child`/`TreeNode` -- are `fdu-7rwf`, which
  this unblocks.

  THE POLARITY, settled from the spec rather than guessed. This bead's summary line says "a
  plane for entries carrying that tag" and its correction paragraph says the stored side is
  the one whose mtime a consumer reads. The spec is unambiguous: "restricted to entries not
  carrying the tag". Those reconcile -- for `gitignore` the untagged side *is* the unignored
  side, which is what a browser shows -- so there is one uniform rule and no per-rule
  polarity to get wrong. A plane holds the entries WITHOUT its tag.

  WHAT LANDED.

  `Promoted(TagId)` and `TagRules::with_promoted`. Promotion moves the tag-rules fingerprint;
  rebinding control files does not, and that difference is exactly the distinction between
  what a rule *reads* and what it *is*. A snapshot written without a plane cannot be
  reinterpreted as one with an empty plane: those say different things -- "nothing was outside
  the tag" and "nobody was counting". An unpromoted set still fingerprints exactly as before,
  so no existing cache is discarded to express "still no planes".

  Promoting a name that is not enabled is refused rather than ignored. A caller who promoted a
  typo would otherwise get every plane silently empty, with no way to tell that from a tree
  where the tag matched nothing.

  `InternedRollUp.planes`, a sorted association list by `Promoted` -- the `by_group`
  precedent, for the same reason: a handful of promoted rules, so a linear scan beats a node
  per key. Empty and unallocated when nothing is promoted, which is the default and the only
  shape the hot path had before.

  The tally arithmetic is now shared. Extracting `merge_tallies` / `unmerge_tallies` and their
  group equivalents means a plane and the totals cannot drift apart by one of them being
  edited and the other not -- they are the same code, not the same idea written twice.

  `recompute_newest_upward` repairs every plane's maximum beside the whole-subtree one, from
  the same pass over the same children. Its early exit had to change: with several maxima at a
  level, one of them settling says nothing about the others, so it now continues while *any*
  of them moved.

  TESTS, mutation-checked.

  `a_plane_and_its_complement_account_for_everything` walks insert, modify, remove-tagged,
  remove-untagged, and the remove-and-reinsert a kind change becomes, asserting at every
  directory that the plane is within the totals field by field. The arithmetic is trivially
  right; the bookkeeping is what can drift, since a plane is merged and un-merged along the
  same ancestor chain and can come apart at any level. Mutation: skipping the plane's unmerge
  gives "plane bytes exceed the total".

  `the_derived_side_reports_no_mtime_because_a_maximum_cannot_be_subtracted` puts the newest
  file on the tagged side, so the whole tree's newest and the plane's newest differ and no
  subtraction recovers the former from the latter.

  NOT MEASURED, and worth being explicit. This adds per-plane state to the ancestor-merge
  path -- the one exp-064 took from 43.73% to 14.07% and campaign 2 plans to delete rather
  than tune. The cost is why promotion is a declared subset rather than a property of every
  tag, and the default is unpromoted, so nothing pays for it unasked. A `make perf-compare`
  run with a promoted rule against a real tree belongs with `fdu-7rwf`, where the feature
  first becomes reachable from a surface.
resolution: null
duplicate_of: null
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
