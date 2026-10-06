---
type: is
id: is-01m480tfdhvzwb9180krvyydpe
title: "PR #177 A4: held-basis refusals suggest a remedy the route does not have"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m480t2frm92ses02tfr36tgv
hold: null
hold_until: null
created_at: 2026-10-06T07:11:05.648Z
updated_at: 2026-10-06T09:04:27.783Z
started_at: 2026-10-06T07:40:01.259Z
closed_at: 2026-10-06T09:04:27.782Z
close_reason: "Fixed in f7cb2ae3: BasisHolder-worded remedies, Request::read_opened; smoke.py asserts opened-root text"
resolution: null
duplicate_of: null
---
Low. opened root (opened.rs:224-240, opened.py:1443-1466), held sort refusal (query_request.rs:1094-1100), Request::new + prepare_report with no index (execution.rs:615, query_request.rs:602,:703,:1226-1231). PR #177, review: https://github.com/jlevy/fdu/pull/177#issuecomment-6011241855
