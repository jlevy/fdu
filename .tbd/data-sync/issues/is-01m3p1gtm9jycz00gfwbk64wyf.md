---
type: is
id: is-01m3p1gtm9jycz00gfwbk64wyf
title: Review the 0.2.2 Linux performance plan and map every performance attribute to hypotheses for an overnight loop
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
hold: null
hold_until: null
created_at: 2026-09-29T07:36:58.249Z
updated_at: 2026-09-29T16:28:58.557Z
started_at: 2026-09-29T07:37:02.176Z
closed_at: 2026-09-29T16:28:58.557Z
close_reason: "Plan, review and overnight loop done: six changes accepted (H171, H175, H172+H176+F6e, H180, H169 phase 1, H183), four hypotheses rejected, 16 cells recorded (exp-175-186, 192-195); draft PR jlevy/fdu#161."
resolution: null
duplicate_of: null
---
Senior review of PR #157/#158's plan (plan-2026-09-29-linux-parity-0.2.2.md, the design study, the pdu brief) against the whole performance-loop record (registry H1-H173, ledger exp-1..191, runbook, campaign-2 plan, floor report) and against peer source checked out in attic/ (pdu, diskus, dut, bfs, gdu, dua, dust, ripgrep ignore, fastwalk, GNU du, ncdu). Output: an attribute-to-hypothesis map with pre-registered accept rules, and a sequenced overnight queue. Then a Fable deep technical review folded in.
