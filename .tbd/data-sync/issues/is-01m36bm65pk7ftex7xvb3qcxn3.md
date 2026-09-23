---
type: is
id: is-01m36bm65pk7ftex7xvb3qcxn3
title: Low findings from the 2026-09-22 alpha stack review
kind: task
status: open
priority: 3
version: 1
labels:
  - stack-followup
dependencies: []
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:43.221Z
updated_at: 2026-09-23T05:25:43.221Z
---
Non-blocking items, each verified in source by the reviewer. #112: R112-4 dead document_raw_words state in query_report.rs; R112-5 --view languages lines-only silently switches the share column to raw words; R112-6 flat coverage/analyzed_files are view-selected; metric independence test covers only ViewSpec::Types (add Families, Languages, Documents). #113: R113-4 PLAUSIBLE summary fast path (TreeStatus::of_walk) does not normalize duplicate walk errors; R113-5 Python watch/since dicts are hand-built (no path_raw; since uses invalidate_subtree); R113-6 Python models never check the schema string. #114: R114-2 merged error list withdraws listings of a sibling stale-only subtree; R114-3 RefreshResult omits retry_required; R114-5 reconcile_epoch Option enforced by expect; R114-6 watch_persistence test relies on sleep(1500) vs 1s throttle. #115: R115-4 dead branches (refresh Verify::None, cache-only fallback arm, cli run rebuilding plan); R115-5 write-policy test restates the formula; R115-6 Python refresh raises after index advanced on persist failure; R115-7 type-rules refusal rebuilds the whole index to validate checksum. #117: R117-4 PLAUSIBLE explicit Format::Text renders in the report format; R117-5 no golden for the stderr bound note under flat formats; R117-6 flat-row tiebreak by native path bytes differs across platforms; check-portability.mjs never reads crates/fdu-core/tests/golden.
