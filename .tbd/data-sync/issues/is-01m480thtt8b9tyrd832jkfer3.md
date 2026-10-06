---
type: is
id: is-01m480thtt8b9tyrd832jkfer3
title: "PR #177 B2: path independence proves view/analyzer sidecar sharing only via --stale-ok"
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
created_at: 2026-10-06T07:11:08.121Z
updated_at: 2026-10-06T09:04:29.951Z
started_at: 2026-10-06T07:40:03.725Z
closed_at: 2026-10-06T09:04:29.950Z
close_reason: "Fixed in f2cc1f64: view warmers and auto->auto revalidated leg; subset 3170 cases allowed"
resolution: null
duplicate_of: null
---
Low. tests/path_independence/matrix.py:193-204 WARMERS; runner.py:705-759 phase_implied. PR #177, review: https://github.com/jlevy/fdu/pull/177#issuecomment-6011241855
