---
type: is
id: is-01m480tfsyd6kq6cfqtyqp0v09
title: "PR #177 A5: the not-shown note is all-or-nothing; partly shown analysis goes unremarked"
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
created_at: 2026-10-06T07:11:06.045Z
updated_at: 2026-10-06T09:04:28.094Z
started_at: 2026-10-06T07:40:01.601Z
closed_at: 2026-10-06T09:04:28.093Z
close_reason: Fixed in 598badf5 (ViewSpec::shows, per-analyzer note) and golden 9d9515f2
resolution: null
duplicate_of: null
---
Low. crates/fdu-core/src/query/query_report.rs:1432-1455. --analyze code --view documents shows no note that code analysis is unshown. PR #177, review: https://github.com/jlevy/fdu/pull/177#issuecomment-6011241855
