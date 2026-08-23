---
type: is
id: is-01m0prgyer27hqzdm2pvjx44qg
title: "Partitioned tallies: tag rules and per-plane roll-ups in the engine"
kind: feature
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prgyv1eq0g0mzgntn1p4n6
  - type: blocks
    target: is-01m0ptezmtmkn04mh1f1rwgdxb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:53.944Z
updated_at: 2026-08-23T17:01:01.036Z
---
Opt-in tag configuration on ScanOptions: compiled gitignore (ignore-crate matcher, correct negation) and hidden-with-allowlist rules; entry tag bits; per-plane roll-up state (files, dirs, bytes, allocated, newest mtime, per-extension) through merge_upward, refresh, and watch re-tagging; .gitignore-edit escalation to InvalidateSubtree; enabled rule set versions the snapshot fingerprint. Builds on the closed spike fdu-p35d (0.39-1.76 us/entry measured). Partition-sum property tests and fingerprint-invalidation tests land with it.

## Notes

IMPLEMENTATION MAP. STATE: index.rs holds two rollup types — public RollUp (:111, by_ext: BTreeMap<String, ExtTally>) and hot-path InternedRollUp (:133, keyed by ExtId). Planes add parallel totals to both, maintained by the same functions: Entry (:214) gains tag bits beside source: Source, already a one-byte discriminant (the padding the provenance design identified); InternedRollUp::merge/::unmerge extend to the new fields so merge_upward (:1499) and unmerge_upward (:1511) need no structural change; plane sums ARE invertible, newest_mtime_ns is not and already has its repair path in recompute_newest_upward (:1525), which extends per plane; contribution (:1466) decides what an entry contributes to each plane. INVALIDATION IS ALREADY WIRED — no new field, no format bump: ScanScope (engine_contract.rs:137) already carries ignore_rules_fingerprint, populated from IGNORE_RULES_FINGERPRINT = 0 at scan.rs:62 under the comment 'No ignore rules exist yet', already serialized by put_scope (snapshot.rs:657) and read by read_scope (:676). Tag rules populate that constant; plane state bumps REDUCERS_FINGERPRINT (scan.rs:65). This invalidates precisely the snapshots recorded under different rules, where adding a mix() to engine_fingerprint (snapshot.rs:171) would have invalidated every snapshot everywhere. DEPENDENCY: the tag matcher wants the ignore crate, and query_glob.rs:7-14 already records the decision and the escape hatch — the module exists to avoid globset's transitive weight for query-time patterns and states that 'if the pattern language grows toward regexes or real gitignore semantics ... globset/ignore goes through the dependency cool-off and this module is deleted'. Real gitignore semantics is exactly this bead, so the hatch applies as written, subject to the 14-day cool-off. TESTS: partition sums as seeded loops in index.rs mod tests (no proptest dependency in this repo) across scan, refresh and watch mutation; plane-equals-all when nothing is tagged; fingerprint invalidation as a snapshot round-trip under changed rules.
