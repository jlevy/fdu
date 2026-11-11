---
type: is
id: is-01m74c9197sj2fzrdhc5jgt126
title: "PR #209 F2: opened read observes capacity under the index read guard"
kind: bug
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m74c8yycvgvjfyne7k65v8gv
created_at: 2026-11-11T04:01:24.775Z
updated_at: 2026-11-11T06:09:29.627Z
closed_at: 2026-11-11T06:09:29.626Z
close_reason: "Fixed in 70f82587: opened read observes before its guard, report_work charges two passes, Python observations detached. Dispositions: https://github.com/jlevy/fdu/pull/209#issuecomment-6788495799"
resolution: null
duplicate_of: null
---
High. opened/read.rs:32,196-198,258-259; query_volumes.rs:207-233; fdu-py lib.rs:551 (no py.detach); report_work pass count. Review F: https://github.com/jlevy/fdu/pull/209#issuecomment-6786513339
