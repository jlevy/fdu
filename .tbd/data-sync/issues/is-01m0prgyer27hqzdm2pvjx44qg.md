---
type: is
id: is-01m0prgyer27hqzdm2pvjx44qg
title: "Tag model foundation: rules, tiers, entry bits, and the tag_rules fingerprint"
kind: feature
status: open
priority: 1
version: 13
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prgyv1eq0g0mzgntn1p4n6
  - type: blocks
    target: is-01m0ptezmtmkn04mh1f1rwgdxb
  - type: blocks
    target: is-01m0t5szzjt8kr7yqkzg78cxhm
  - type: blocks
    target: is-01m0t5t1ghzmetfs4qjbrzx44r
  - type: blocks
    target: is-01m0t5t2sa2rn3qm3m4dycv7hv
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:53.944Z
updated_at: 2026-08-24T15:23:02.626Z
---
The generic half of partitioned tallies, restructured 2026-08-24 after the owner
directed that gitignore be one flag among several (text, binary, and other future
facts), not the feature's name. The gitignore rule is fdu-brt0, promotion is fdu-pxfz,
hidden admission is fdu-xyvu, surfaces remain fdu-7rwf. This bead is the model, and it
carries no new dependencies.

THE MODEL. A tag is a named boolean fact about an entry, produced by an enabled TagRule
and stored as one bit beside ext_id and group_id. Rules carry a TIER declaring what
they may read -- Name (basename), Path (relative path plus walk-scoped control files),
Content (file bytes) -- and Content-tier rules are REJECTED at enable time in v1 with
an error naming the cost class, so nobody silently turns a metadata walk into a content
walk. Tags are unbounded and cheap; PLANES are not tags -- a plane is a maintained
aggregate for a rule explicitly promoted, and promotion is fdu-pxfz. Filtering by an
unpromoted tag re-aggregates by walking, the two-tier rule Selection already applies to
every other predicate.

Categorical facts (mime type) are NOT tags: they are interned-key tally maps, the
mechanism ext_id/group_id already use. Two shapes; neither absorbs the other.

WHAT LANDS HERE:
- engine_contract.rs: ScanScope.ignore_rules_fingerprint RENAMED tag_rules_fingerprint.
  Same wire position, and an empty rule set still fingerprints to 0, so every existing
  snapshot stays valid. scan.rs's IGNORE_RULES_FINGERPRINT constant follows and reads
  the enabled set.
- A TagRules registry: TagId, TagBits (u32), TagTier, TagRule, fingerprint, indexed.
  Engine-declared closed set in v1; runtime-supplied glob rules are future work the
  TypeRegistry precedent already shapes.
- Entry.tag_bits computed at APPLY TIME by an index-held evaluator -- the same place
  and the same reason as ext_id/group_id: one computation site, so a watch upsert is
  tagged identically to a scan upsert. NO snapshot format bump: bits are recomputed at
  load exactly as ext_id/group_id are, and for Path-tier rules that recomputation reads
  the CURRENT control files, which is fresher than anything a snapshot could carry.
- First rule, zero dependencies: `dotfile` (Name tier, basename starts with '.').
  Available, not enabled by default, never promoted. It exists to prove the model
  end-to-end -- goldens, parity, watch -- before the gitignore dependency lands; the
  scripted watch backend is the precedent for shipping the seam the tests use. It is a
  TAG (filter, both numbers visible), not the hidden-path PRUNE the reconciliation
  demoted (fdu-xyvu) -- different axes, and the spec states the distinction.
- Selection gains a tag predicate (require/exclude by rule name; exact CLI spelling
  decided in-bead), served by re-aggregation. Scope axis --tags enables rules.
  ScanOptions.tags in Python; Child rows carry their tag names.
- Goldens with the dotfile rule in every format, replayed by the parity harness.

The partition property (tagged + complement = all) is stated per tag from day one and
tested via re-aggregation here; the maintained version arrives with fdu-pxfz.

Blocked by nothing. Blocks fdu-brt0, fdu-pxfz, and fdu-7rwf.

## Notes

RESTRUCTURED 2026-08-24 from "Partitioned tallies: tag rules and per-plane roll-ups"
into the tag-model foundation, at the owner's direction that the model be generic and
gitignore be one rule among several. The genericity review that preceded this (three
couplings: tags 1:1 with planes, no tier on a rule, the fingerprint named for one
policy) is applied in the description; the split beads are fdu-brt0 (gitignore rule +
the feature-gated dependency, with the measured evidence and the MSRV pin), fdu-pxfz
(promotion/planes), fdu-xyvu (hidden admission as scope), fdu-n7mv (Classification
flags fold-in). Surfaces remain fdu-7rwf.

The closed spike fdu-p35d (0.39-1.76 us/entry) remains the matcher evidence for the
gitignore rule.
