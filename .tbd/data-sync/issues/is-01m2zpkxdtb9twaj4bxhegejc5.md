---
type: is
id: is-01m2zpkxdtb9twaj4bxhegejc5
title: "Restore test guards dropped by the #91/#92 fix-consolidation merges (R3 name negatives, H138 sharing)"
kind: task
status: closed
priority: 2
version: 4
labels: []
dependencies: []
created_at: 2026-09-20T15:23:07.575Z
updated_at: 2026-09-21T08:20:31.393Z
closed_at: 2026-09-21T08:20:31.393Z
close_reason: "Shipped on #104 (https://github.com/jlevy/fdu/pull/104) at 743d2b9c / 4d78558e vs main a290aedc. CI run 35576573158 green on ubuntu/macos/windows including Performance evidence. exp-116 change_pct is the paired −2.158%; commit quoted as 984e4618; digest/regime/errata corrected; ledger+report regenerated. R3 name negatives, valid-rename load control, and H138 sharing guard restored; apply timer starts at candidates.remove. peak_rss_bytes prints as bytes (exp-117 377.5→339.4 MiB). Local rust-test passed except the known parallel_equivalence flake (not fixed)."
---
From the independent pre-merge verification of PRs #91 and #92 (2026-09-20). The merges that consolidated two concurrent fix lines (870bdcfb, 3cc94898/937f9445) kept one side's tree; nothing incorrect shipped, but some guards were lost.

- V92-1: `snapshot_names_must_equal_their_single_normal_component` lost its direct negatives for the empty name, `.`, `..`, and non-UTF-8 plus `/`. The public-open alias test lost its positive control. The two retained rejection tests (snapshot.rs ~:1403, ~:1427) have no control proving the forged image is otherwise valid, so a future record-layout change that breaks `entry_record_fields` would let both pass vacuously. Add a case where a valid rename loads and is found by lookup; restore the four negatives.
- V92-2: `unfiltered_multi_view_sharing_builds_every_entry_once` was removed, so H138's sharing has no regression guard: flipping `row_consumers > 1` (query_report.rs ~:1028) to never share passes every test at head. Add a two-view case to `tests/query_allocations.rs` asserting `[Types, Families]` allocates less than 2x `[Types]` (measured: 5,134 vs 8,214 vs 10,268 unshared).
- V92-4: the allocation guard's `FILES * 3` threshold assumes two walk allocations per file; if the walk gets cheaper a reintroduced clone passes. Tie the bound to a measured single-view baseline instead.
- V91-8: `bottom_up_rebuild_matches_incremental_nested_rollups` uses only `CoverageReason::Analyzed` records, so `by_type`/`by_family` merging for non-analyzed records at nested directories is proven only by reading. The R1 timer test exercises the helpers, not `load_content_cache`'s use of them. The kept rollback tests use the lines analyzer only rather than `AnalysisSet::ALL`.
- Dead code: `apply_analysis_record`'s `update_rollups = false` branch (index.rs ~:3622).

## Notes

Context posted for Linux handoff on PR #94: https://github.com/jlevy/fdu/pull/94#issuecomment-5757234985
