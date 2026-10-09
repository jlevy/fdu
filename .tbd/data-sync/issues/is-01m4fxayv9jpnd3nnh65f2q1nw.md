---
type: is
id: is-01m4fxayv9jpnd3nnh65f2q1nw
title: Measure the age column's cost on the default report
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T08:44:06.888Z
updated_at: 2026-10-09T10:11:52.044Z
---
Two regimes: (1) paired make perf-compare of the default one-shot report on a real tree, cold and warm, control = ~/fdu-perf/control-148ef78e/perf_probe; (2) a retained-index report timing (Index.report / opened root) at a large entry count, before and after. If (2) regresses measurably, switch to a per-directory activity maximum maintained beside newest_mtime_ns (snapshot fingerprint bump). Record regime and result.

## Notes

Phase 1 implementation landed on claude/tree-age-column (fac3d744 engine). What to measure: an unfiltered tree (default fdu PATH, Index.report, opened-root and watch repaints) now runs query_subtrees::activity: one iterative post-order over entry ids (no paths), each directory pushed twice and its children read once (kind_of for every child, attrs_of for non-directories, the roll-up's newest file per directory), into a dense Vec<Option<DirectoryActivity>> sized to Index::slots() (~24 B per arena slot, freed after the report). O(entries) time, O(slots) transient memory. It replaces measure() for unfiltered trees over partial indexes (so a partial unfiltered tree does no more passes than before). Filtered trees add no pass: activity folds into walk's existing per-directory map entry. Opened-root report pricing (opened/read.rs report_work) still charges one entries pass for a tree view, so the new pass is inside that charge only because expansion is cheaper than a pass; repricing is a separate decision. Not yet measured: paired make perf-compare and a retained-index timing.
