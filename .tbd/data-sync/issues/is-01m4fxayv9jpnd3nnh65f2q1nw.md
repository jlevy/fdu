---
type: is
id: is-01m4fxayv9jpnd3nnh65f2q1nw
title: Measure the age column's cost on the default report
kind: task
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-09T08:44:06.888Z
updated_at: 2026-10-09T12:38:31.320Z
started_at: 2026-10-09T11:28:26.350Z
closed_at: 2026-10-09T12:38:31.319Z
close_reason: "Measured, and took the spec's fallback. Regime: macOS, Apple M1 Pro, uncontrolled host (load_1m 20-50 on 10 CPUs), ~/.rustup 77,355 entries, paired make perf-compare, 12 trials, control 148ef78e. Phase-1 pass (run tree-age-column): default-tree wall -2.07% [-12.2, +9.6] no change but peak RSS +10.4% regressed; index-second-report component 0.128 -> 0.789 ms (+496%); opened-second-report 0.252 -> 4.92 ms (+1545%). Diagnosis: opened roots keep children in name-keyed BTreeMaps (all 3,427 dirs) whose order does not follow the arena (72,035 of 77,354 children non-contiguous vs 0 in the detached index; slots = live entries), so the pass is 2-3x the detached index's (1.45 vs 0.74 ms warm, 1.93 vs 0.82 ms after a cache sweep); the rest of the paired 4.9 ms was host noise. Fallback (10ae731f): per-directory newest activity maintained beside newest_mtime_ns in the all partition only, read per row by an unfiltered tree over a complete index with no scan depth; the pass remains for partial or depth-bounded indexes. No snapshot format or fingerprint change: roll-ups are rebuilt from entry records on load. After (run tree-age-rollup, same command): default-tree wall +3.37% [-11.8, +13.1] no change, peak RSS +0.78% [-4.3, +5.8]; index-second-report component 0.129 -> 0.134 ms (+5.8% [+2.7, +7.5], ~5 us per-row column work); opened-second-report component 0.135 -> 0.153 ms (+13.7% [+11.7, +14.7]), wall +1.56% [+1.06, +3.75] (recheck: wall -0.80% [-3.83, +2.54], component 0.132 -> 0.154 ms); cold-scan-index wall -0.64% [-5.0, +2.9]; warm-snapshot-load wall -0.24% [-3.2, +0.8], component +2.21% [+0.53, +2.71] noninferior at +3% (recheck +1.73% [-5.3, +14.0]). Tests 27aea45a; make check passes."
resolution: null
duplicate_of: null
---
Two regimes: (1) paired make perf-compare of the default one-shot report on a real tree, cold and warm, control = ~/fdu-perf/control-148ef78e/perf_probe; (2) a retained-index report timing (Index.report / opened root) at a large entry count, before and after. If (2) regresses measurably, switch to a per-directory activity maximum maintained beside newest_mtime_ns (snapshot fingerprint bump). Record regime and result.

## Notes

Phase 1 implementation landed on claude/tree-age-column (fac3d744 engine). What to measure: an unfiltered tree (default fdu PATH, Index.report, opened-root and watch repaints) now runs query_subtrees::activity: one iterative post-order over entry ids (no paths), each directory pushed twice and its children read once (kind_of for every child, attrs_of for non-directories, the roll-up's newest file per directory), into a dense Vec<Option<DirectoryActivity>> sized to Index::slots() (~24 B per arena slot, freed after the report). O(entries) time, O(slots) transient memory. It replaces measure() for unfiltered trees over partial indexes (so a partial unfiltered tree does no more passes than before). Filtered trees add no pass: activity folds into walk's existing per-directory map entry. Opened-root report pricing (opened/read.rs report_work) still charges one entries pass for a tree view, so the new pass is inside that charge only because expansion is cheaper than a pass; repricing is a separate decision. Not yet measured: paired make perf-compare and a retained-index timing.
