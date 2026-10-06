---
type: is
id: is-01m480tdqabng3pnpvkh3kpfct
title: "PR #177 B1: cross_warm.py takes the analyzer set from argv; cross-warm pass fails"
kind: bug
status: in_progress
priority: 2
version: 2
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m480t2frm92ses02tfr36tgv
hold: null
hold_until: null
created_at: 2026-10-06T07:11:03.913Z
updated_at: 2026-10-06T07:11:11.756Z
started_at: 2026-10-06T07:11:11.755Z
---
High. tests/correctness/cross_warm.py:50,:90-92,:134; warm_cold.py:148. W_lines -> a_lines_documents NOT-WARM(scanned), exit 1; W_words -> a_lines_documents unchecked. Read the effective set from the engine (request.analyze), add view-only asks/warmers. PR #177, review: https://github.com/jlevy/fdu/pull/177#issuecomment-6011241855
