---
type: is
id: is-01m2ey0wqt2v7fcmzz0hptxebz
title: Name the extension level on each Python field that carries an extension
kind: task
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-14T03:05:27.539Z
updated_at: 2026-09-14T03:54:45.144Z
closed_at: 2026-09-14T03:54:45.144Z
close_reason: "d48b8f8: RollUp.by_extension and ExtensionRow.extension say raw, NameClassification.logical_extension says logical and canonical_extension says canonical, EntrySelection.logical_extensions says it matches the logical level, and terminal_extensions says it is none of the three. Each uses file.c++ and release.v2.zip and points at Classification and Extension Levels in the engine architecture document."
resolution: null
duplicate_of: null
---
Split from fdu-eupp. fdu-eupp stated the three extension levels (raw, File Rollup logical, canonical match) in the `classify` module documentation and the engine architecture document. It left the Python half alone, because another agent owned the Python surface during that wave.

The Python fields that carry an extension do not say which level they are:

- `crates/fdu-py/python/fdu/_models.py`: `by_extension` on the tally model (line 316 at 855d300). This is the raw level, keyed by `classify::ext_bucket`, so `file.c++` is `.c++` and `Makefile` is `(none)`.
- `crates/fdu-py/python/fdu/opened.py`: `logical_extension` and `canonical_extension` on the name classification (lines 386-387 at 855d300). These are the logical and canonical levels, from `NameClassification`.
- `crates/fdu-py/python/fdu/opened.py`: `logical_extensions` on the selection (line 476 at 855d300). A selection matches the logical level, not the raw one.

Fix: give each field a docstring or attribute comment that names its level and links the table in the engine architecture document's "Classification and Extension Levels" section. Use `file.c++` and `release.v2.zip` as the examples, since those are where the levels part. Do not copy the table.
