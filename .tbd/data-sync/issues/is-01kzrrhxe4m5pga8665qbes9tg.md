---
type: is
id: is-01kzrrhxe4m5pga8665qbes9tg
title: "PR #4 review R2: apply constant-RSS detection to record"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01kzrrhm8bnv31bc5rqdex8yfp
created_at: 2026-08-11T15:55:12.707Z
updated_at: 2026-08-11T16:00:02.845Z
closed_at: 2026-08-11T16:00:02.844Z
close_reason: Moved run-wide degenerate resource detection into the shared decision layer and applied it to both record and environment matrices; added a regression proving constant RSS blocks acceptance and passed make check.
---
PR #4 thread PRRT_kwDOTx0nJs6YKqAp at benchmarks/realtree/record.py:223-234 and environment.py:679-689. record must fail closed on a non-discriminating run-wide peak_rss_bytes signal exactly as the environment matrix does. Add a focused failing test and share the detection logic.
