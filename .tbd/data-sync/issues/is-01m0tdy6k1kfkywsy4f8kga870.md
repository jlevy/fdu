---
type: is
id: is-01m0tdy6k1kfkywsy4f8kga870
title: Warm snapshots rebuild derived state under the wrong registry
kind: bug
status: open
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels:
  - pr47-review
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
  - type: blocks
    target: is-01m0tra5gw0ap6nbbzgt7egvr4
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:51.648Z
updated_at: 2026-08-24T20:45:43.135Z
---
At PR 47 head e658915, snapshot::load materializes entries before the caller registry is attached. insert_loaded_child therefore recomputes canonical extension and group state with TypeRegistry::compiled, while with_types only swaps the registry pointer after derived rollups already exist. An unchanged warm reconciliation never rebuilds them, so a custom registry can return default-registry extension or group totals indefinitely. The same late-attachment path calls retag, whose traversal joins one PathBuf per entry even for Name-tier rules, reversing the loader optimization called out in the PR. Fix: make snapshot materialization accept the validated active TypeRegistry and TagRules after checking header fingerprints, install them before inserting entries, and derive every entry and rollup exactly once. Tests: cold, warm, and cache-only opens under a deliberately different compound-extension and group registry must agree; a Name-tier tag warm load must not reconstruct paths per entry. Review finding FDU47-R1.

## Notes

DESIGN SETTLED (2026-08-24 review). Verified against the code: `snapshot::load` builds
`Index::new_with_scope` (compiled registry, empty tag rules), materializes every entry
through `insert_loaded_child` -- deriving ext_id/group_id under the compiled registry --
and only then does `open_for_report` swap pointers (`with_types`) and re-traverse
(`with_tag_rules` -> `retag`, one PathBuf join per entry even for Name-tier rules).

Also found, unnamed by the review: the root/scope filter runs AFTER full
materialization, so a mismatched snapshot pays the entire load before being discarded.

THE FIX. Two-phase load: `snapshot::peek_header` (root, scope fingerprints) -> validate
against the request -> only then materialize, with the caller's `TypeRegistry` and
`TagRules` installed on the index BEFORE the first entry streams in. `insert_loaded_child`
already derives lazily and correctly once the rules are present; `retag` then has no
load-path caller left (keep it for the watch rebind path). For Path-tier tags at load,
replace the closure's per-record `path_of(parent)` ancestor walk with a dir-path table
(parents precede children in the stream, so the loader can carry EntryId -> PathBuf for
directories only). Name-tier must not construct paths at all -- prove it with the
structural counter R1 asked for.

Interaction with fdu-0778: binding gitignore matchers from the loaded index (not a tree
walk) needs the structure first -- so the load order becomes: header -> validate ->
install types -> materialize entries -> bind Path-tier matchers from the index's own
control-file entries -> evaluate tags (one pass, dir-path table). That single ordering
resolves both beads' load halves.

TESTS. Cold/warm/cache-only agreement under a deliberately custom compound-extension and
group mapping (the R1 fixture); a counter proving a Name-tier warm load reconstructs no
paths; a counter proving a mismatched-scope snapshot is rejected from the header alone.
