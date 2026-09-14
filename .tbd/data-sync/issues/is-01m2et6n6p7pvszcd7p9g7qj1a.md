---
type: is
id: is-01m2et6n6p7pvszcd7p9g7qj1a
title: "PR #48 review FIX48-6: a Continue racing close() reads ContinuationUnavailable, not OpenedIndexClosed"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2et5s9yyv9e6tz783rp2m6c
created_at: 2026-09-14T01:58:42.133Z
updated_at: 2026-09-14T02:49:16.053Z
closed_at: 2026-09-14T02:49:16.051Z
close_reason: "Fixed in 9fab783: ContinuationTable::take checks closed first and returns OpenedIndexClosed. https://github.com/jlevy/fdu/pull/48#issuecomment-5658305981"
resolution: null
duplicate_of: null
---
Low, from 1e8706f. At f917cb7: read.rs:102-111; continuation.rs:134-148 take, 167-171 close. read() releases the lifecycle guard after its phase check, so a Continue whose take() runs after shutdown closed the table finds it empty and returns ContinuationUnavailable, while a fresh page in the same race returns OpenedIndexClosed. Fix: take() checks closed first. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5193206420
