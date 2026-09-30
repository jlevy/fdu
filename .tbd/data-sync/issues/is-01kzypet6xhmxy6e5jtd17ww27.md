---
type: is
id: is-01kzypet6xhmxy6e5jtd17ww27
title: Bound content-analysis candidate scheduling instead of materializing every path
kind: bug
status: closed
priority: 2
version: 5
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-08-13T23:14:00.541Z
updated_at: 2026-09-30T04:06:19.570Z
closed_at: 2026-09-30T04:06:19.570Z
close_reason: "Fixed in 2b67fd5c: an analysis pass walks the index in batches of 4,096 candidates over one resumable AnalysisWalk (Index::next_analysis_candidates), counts the pending candidates first without building any (count_pending_analysis_candidates) so the progress denominator stays exact, and applies each batch's results conditionally on revision and fingerprint as before. Tests: analysis_candidates_come_in_bounded_batches_in_walk_order (every batch size gives the whole walk's order, each batch bounded), batches_skip_what_the_content_tier_already_holds, and the differential scheduling_in_batches_of_one_leaves_the_same_records_as_all_at_once (records, counts, coverage, bytes read identical at batch 1 and batch MAX). Measured on linux-v6.12 (86,643 files) with --analyze lines --view summary --cache off --workers 4, release builds before/after, two runs each: peak RSS 114.0 MiB to 84.4 MiB (about 340 B per candidate no longer held at once), wall 2.85 s both. CHANGELOG Fixed entry added."
resolution: null
duplicate_of: null
---
Index::analysis_candidates clones every regular-file path into a Vec before work enters the bounded worker channel. Reader concurrency is bounded, but scheduling memory remains O(files). Stream candidates through bounded scheduling while preserving conditional apply semantics.
