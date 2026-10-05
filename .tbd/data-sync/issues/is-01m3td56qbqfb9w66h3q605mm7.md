---
type: is
id: is-01m3td56qbqfb9w66h3q605mm7
title: "PR #170 review R12: the tests cover few failure paths"
kind: bug
status: closed
priority: 3
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:18.057Z
updated_at: 2026-10-01T03:26:34.178Z
started_at: 2026-10-01T00:17:37.496Z
closed_at: 2026-10-01T03:26:34.177Z
close_reason: "Fixed in df829ec8 and 9e53c996: a test with each fix. PR #170, disposition map https://github.com/jlevy/fdu/pull/170#issuecomment-5924110534"
resolution: null
duplicate_of: null
---
tests/release/test_stability_pass.py lacks tests for: a partially caught break, cross-lint skip lines, a rerun over a failed record, a harness that exits 0 after skipping a phase, a private path in a failure detail, a caller's --target-dir, a candidate whose --version names another commit, and the pty probe exiting 2. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
