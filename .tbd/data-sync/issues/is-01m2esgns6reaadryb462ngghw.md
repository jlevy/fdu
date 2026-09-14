---
type: is
id: is-01m2esgns6reaadryb462ngghw
title: Document the public extension API's raw and File Rollup levels, and pin the raw level with a golden
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-14T01:46:41.830Z
updated_at: 2026-09-14T01:46:41.830Z
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
