---
type: is
id: is-01m0pt9j1mbn3za7fyym7pyr9t
title: Bounded per-directory extension and filename rows with a remainder
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:02:49.012Z
updated_at: 2026-08-23T19:38:24.901Z
closed_at: 2026-08-23T19:38:24.901Z
close_reason: "RollUp.by_ext gains a Bound: rollup_bounded/total_bounded/children_bounded keep the largest N rows by apparent bytes (ties by name) and aggregate the rest into ext_remainder. Applied before names are cloned, so a wide subtree costs one pass plus N clones. Reached from Python as total/rollup/children(extensions=N). Bound moved to engine_contract so depth, row limits, and extension rows share one vocabulary."
resolution: null
duplicate_of: null
---
RollUp.by_extension is an unbounded map per directory. Metabrowser bounds its equivalents (ext_top, filename_top, remaining_top) because a browser shows a handful of rows while a wide tree has many. Add a bound with a stated remainder aggregate, same contract as the TreeNode remainder: truncate freely, never silently.

## Notes

RollUp.by_ext (index.rs:111) is an unbounded BTreeMap per directory, resolved from the ExtId-keyed InternedRollUp (:133) by named_rollup (:1442). The bound and its remainder follow the same contract as the TreeNode remainder in fdu-knyw.
