---
type: is
id: is-01m36bm65pk7ftex7xvb3qcxn3
title: Low findings from the 2026-09-22 alpha stack review
kind: task
status: open
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:43.221Z
updated_at: 2026-09-23T07:41:46.849Z
---
Non-blocking items, each verified in source by the reviewer. #112: R112-4 dead document_raw_words state in query_report.rs; R112-5 --view languages lines-only silently switches the share column to raw words; R112-6 flat coverage/analyzed_files are view-selected; metric independence test covers only ViewSpec::Types (add Families, Languages, Documents). #113: R113-4 PLAUSIBLE summary fast path (TreeStatus::of_walk) does not normalize duplicate walk errors; R113-5 Python watch/since dicts are hand-built (no path_raw; since uses invalidate_subtree); R113-6 Python models never check the schema string. #114: R114-2 merged error list withdraws listings of a sibling stale-only subtree; R114-3 RefreshResult omits retry_required; R114-5 reconcile_epoch Option enforced by expect; R114-6 watch_persistence test relies on sleep(1500) vs 1s throttle. #115: R115-4 dead branches (refresh Verify::None, cache-only fallback arm, cli run rebuilding plan); R115-5 write-policy test restates the formula; R115-6 Python refresh raises after index advanced on persist failure; R115-7 type-rules refusal rebuilds the whole index to validate checksum. #117: R117-4 PLAUSIBLE explicit Format::Text renders in the report format; R117-5 no golden for the stderr bound note under flat formats; R117-6 flat-row tiebreak by native path bytes differs across platforms; check-portability.mjs never reads crates/fdu-core/tests/golden.

## Notes

2026-09-23 delta-review Lows added: E-1 (#115 4b40ff4c) refresh: if the content save fails after the metadata save succeeded, persistence_owed stays set and the next pass repeats one metadata write (bounded; clear the debt right after snapshot::save inside persist_index_changes). E-2 (#98 fc352c83) when a Windows root's volume is unavailable (root_dev 0), --one-filesystem is silently unbounded; add a status note. Also deferred with rationale by fixers: R113-6 (schema check in Python models: test and circular-import constraints), R114-5 (reconcile_epoch Option: 15 call sites in scan.rs), R114-6 (watch_persistence sleep: needs a skipped-save signal), R115-4 cli.rs plan rebuild (prepare_report tuple shared with Python), R115-5, R115-6.
