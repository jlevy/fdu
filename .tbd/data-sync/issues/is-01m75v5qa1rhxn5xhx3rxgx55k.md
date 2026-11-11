---
type: is
id: is-01m75v5qa1rhxn5xhx3rxgx55k
title: "PR #209 I2: the Linux host gate's check for skipped tests cannot fail"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m75v586ca3vv6vremva5h5er
hold: null
hold_until: null
created_at: 2026-11-11T17:40:59.329Z
updated_at: 2026-11-11T18:08:50.629Z
started_at: 2026-11-11T17:41:25.543Z
closed_at: 2026-11-11T18:08:50.629Z
close_reason: "Fixed in 52c6db7d: unanchored grep -c 'skipped:' on a log under $GATE; correctness-runbook points at the guide's step 3. Proved: unprivileged (setpriv nobody) 7 passed, count 6 (old anchored check 0); root 7 passed, count 0."
resolution: null
duplicate_of: null
---
Medium. docs/project/guides/linux-host-gate.md:107-113. With --test-threads=1 --nocapture libtest prints 'test NAME ... ' before the test's own 'skipped:' line, so grep -c '^skipped:' prints 0 when six of seven tests skipped. Fix: unanchored grep -c 'skipped:', log under $GATE; point docs/project/guides/correctness-runbook.md:95 at the guide's step. Review: https://github.com/jlevy/fdu/pull/209#issuecomment-6802796047 (PR #209).
