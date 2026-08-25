---
type: is
id: is-01m0prgyer27hqzdm2pvjx44qg
title: "Tag model foundation: rules, tiers, entry bits, and the tag_rules fingerprint"
kind: feature
status: closed
priority: 1
version: 14
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
updated_at: 2026-08-24T17:19:28.069Z
closed_at: 2026-08-24T17:19:28.068Z
close_reason: |
  Shipped as `crates/fdu-core/src/tags.rs` plus wiring across the engine and all three
  surfaces. `make check` is green, which includes the golden corpus replayed against the
  command line and against the Python package.

  WHAT LANDED

  - `tags.rs`: `TagId`, `TagBits` (u32, so `MAX_TAG_RULES` is 32), `TagTier`
    (Name/Path/Content), `TagRule`, `TagRules` with an engine-declared catalogue,
    `from_names`, `mask_of`, `names_of`, and an FNV-1a fingerprint over the enabled names
    and tiers in order. Content-tier rules are refused at enable time with a message
    naming the cost class.
  - `ScanScope.ignore_rules_fingerprint` renamed `tag_rules_fingerprint`, same wire
    position, reading the enabled set. `scan.rs`'s `IGNORE_RULES_FINGERPRINT` constant is
    gone. The empty set still fingerprints to 0, so no existing snapshot is invalidated,
    and a test pins exactly that.
  - `Entry.tag_bits` computed at every insert site: `upsert_beneath`, the snapshot
    loader's direct-insert path, and `ensure_dir_chain`.
  - `Selection.tags: TagFilter { any, none }` with `Candidate.tags`; folded into
    `is_unfiltered()` so a tag filter drops a report off the precomputed tier, which is
    the two-tier rule this bead promised. Served by re-aggregation.
  - `dotfile` ships as the Name-tier reference rule: available, not enabled by default,
    never promoted.
  - Surfaces: `--tag-rules LIST` on Scope with `--tag`/`--not-tag` repeatable on
    Selection; `ScanOptions.tag_rules` and `Selection.tags`/`.not_tags` in Python.
    `FileRow.tags` and `Child.tags` carry the names on both surfaces, in every format.
  - Five golden sessions in `cli-axes.tryscript.md`, all recorded by the parity harness as
    exact matches rather than as a declared deviation class. A `check_tags_are_a_named_
    fact_per_entry` case in `public_smoke.py` pins listing and report agreeing about the
    same entry.

  FOUR DECISIONS MADE AGAINST THE PLAN AS WRITTEN, each forced by something the code made
  visible rather than by argument.

  1. TAG BITS ARE NOT SERIALIZED, AND `with_tag_rules` RE-TAGS. The bead said bits are
     recomputed at load "exactly as ext_id/group_id are". They are not: the snapshot
     loader restores entries BEFORE the caller's rules are adopted, so tagging only at
     insert would leave a warm start answering "no tags" for every entry while a cold scan
     of the same tree answered correctly. One tree, two answers, and it would read as a
     cache fault. `Index::retag()` walks the loaded index from the root when rules are
     adopted. Verified end to end: a `--cache only` run that walks zero files returns the
     same rows as a cold scan.

  2. `TagRules::evaluate` TAKES THE PATH AS A CLOSURE. The upsert path holds a path; the
     snapshot loader holds a parent id and a basename, and reconstructing a path per
     record is exactly the work a callgrind profile put at ~27% of load in the allocator
     and which that path was rewritten to avoid. The first draft of this wiring called
     `path_of(parent).join(name)` per record -- in the function whose own doc comment says
     why it must not. The closure means a Path-tier rule (fdu-brt0) changes `tags.rs`
     alone rather than every insert site.

  3. TWO FLAGS, NOT ONE. Enabling is Scope (`--tag-rules`, matching `--type-rules`) and
     filtering is Selection (`--tag`/`--not-tag`, repeatable and any-of, matching
     `--include`/`--exclude`). Folding them together would make `--not-tag` silently
     invalidate a snapshot, which is the exact scope-versus-selection line the design
     exists to hold. Filtering on a rule that is not enabled is REFUSED, with its own
     error variant: a mask of zero is indistinguishable from no constraint, so the
     permissive reading hands back every entry to a caller who believed they had narrowed.

  4. THE COMMAND LINE ADDS NO `invalid --tag:` PREFIX. The parity harness caught this: the
     library message already quotes the rejected name and lists what is available, so the
     prefix added only the one thing the Python surface could not say identically. Both
     surfaces now print the same sentence.

  Two smaller reversals worth recording. `ensure_dir_chain` tags its placeholder
  directories rather than leaving them at zero -- `apply_upsert` on an existing entry of
  the same kind rewrites attributes and source only, so an ancestor placeholder would have
  stayed untagged for the life of the index, and every ancestor of a deep first
  observation enters there. And the `dotfile` matcher excludes `.` and `..`, which are
  path components rather than dotfiles.

  The partition property is exercised by the golden pair: `--tag dotfile` and
  `--not-tag dotfile` over the same tree return disjoint sets whose union is every entry.
  The maintained version arrives with fdu-pxfz.

  Unblocks fdu-brt0 (the gitignore rule and its feature-gated dependency), fdu-pxfz
  (promotion/planes), and fdu-7rwf (surfaces).
resolution: null
duplicate_of: null
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
