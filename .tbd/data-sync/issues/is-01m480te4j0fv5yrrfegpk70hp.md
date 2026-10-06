---
type: is
id: is-01m480te4j0fv5yrrfegpk70hp
title: "PR #177 A1: CHANGELOG does not name the incompatible changes"
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
created_at: 2026-10-06T07:11:04.337Z
updated_at: 2026-10-06T09:04:26.852Z
started_at: 2026-10-06T07:40:00.187Z
closed_at: 2026-10-06T09:04:26.851Z
close_reason: "Fixed in 8de338d5: Breaking entries for each CLI and Rust change, Added section"
resolution: null
duplicate_of: null
---
Medium. CHANGELOG.md:12-25. Rust: Request.implied_by, RequestError variants removed/reshaped, WatchContent struct variant, suggested_view ViewSpec. CLI: --analyze lines --view documents runs words; --view full under lines/code drops DOCUMENTS. PR #177, review: https://github.com/jlevy/fdu/pull/177#issuecomment-6011241855
