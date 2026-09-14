---
type: is
id: is-01m2esgns6reaadryb462ngghw
title: Document the public extension API's raw and File Rollup levels, and pin the raw level with a golden
kind: task
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-14T01:46:41.830Z
updated_at: 2026-09-14T03:18:26.292Z
closed_at: 2026-09-14T03:18:26.290Z
close_reason: |
  0a2e341: the classify module doc now has an "Extension levels" section. It covers raw (derive_ext, ext_bucket), logical (logical_ext, NameClassification::logical_extension), and canonical (TypeRegistry::canonical_ext, classify_name), with a table over archive.tar.gz, release.v2.zip, file.c++, .gitignore, and notes. under the compiled registry. The values come from logical_canonical_and_raw_extensions_answer_different_questions (notes. added as a row) and the derive_ext/ext_bucket doctests. module_documentation_extension_table_matches_the_functions reads that table back from the source and checks every cell. The engine architecture document has a new "Classification and Extension Levels" section with the same table; before this there was no classification section. The golden: a new tests/golden/fixtures/extension-levels fixture (archive.tar.gz, release.v2.zip, file.c++, notes.md~) and a cli-axes session recording the extensions view. At the logical level three of those rows would bucket differently: (none), (none), and .v2.zip. The parity artifact gained the session's check line. No local build (disk floor); CI green on all jobs: run 34801745317. The Python docstring half is split to fdu-k7lj.
resolution: null
duplicate_of: null
---
Contract follow-up from PR #48 review CLASS-2 (fixed as fdu-d1hj in 3b6de62, option 1), recorded by the fixer.

**What landed.** `derive_ext` is `main`'s raw rule again: any final dotted component, whatever its bytes or length, so `file.c++` gives `.c++`. The File Rollup two-component, eligible-component extension exists only through `logical_ext`, `TypeRegistry::canonical_ext`, and `TypeRegistry::classify_name`. Extension tallies, the extension view, and unknown-type labels use the raw extension.

**The public API now has two extension levels, plus a canonical match.** Each function's rustdoc describes itself, but nothing states the levels together:
- raw: `classify::derive_ext` and `classify::ext_bucket` (`crates/fdu-core/src/classify.rs:906, 934` at f917cb7). These feed the extension view, per-directory tallies, and `unknown:` labels.
- File Rollup logical: `classify::logical_ext` (`classify.rs:875`). This is what a portable row reports.
- canonical match: `TypeRegistry::canonical_ext` and `classify_name` (`classify.rs:563, 588`). This gives the declared extension that matched, or `None` for an undeclared one.

So `file.c++` has raw `.c++`, logical `None`, and canonical `None`, while `release.v2.zip` has raw `.zip`, logical `.v2.zip`, and canonical `.zip`. A library or Python caller choosing between them has to read three doc comments to learn that.

**To do.**
- State the levels once, where a consumer looks: the `classify` module doc, the engine architecture document's classification section, and the Python docs for any field that exposes an extension. Give a table of example names across the levels.
- Pin the raw level with a CLI golden. The disposition notes that "no CLI golden fixture name buckets differently", so a regression that swapped the levels again would pass every golden. The review's option 2 suggested a fixture name with a `~`, `#`, or `+` extension.

Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101
