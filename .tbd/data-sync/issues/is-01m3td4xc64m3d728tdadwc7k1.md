---
type: is
id: is-01m3td4xc64m3d728tdadwc7k1
title: "PR #170 review R6: cleanup can delete a directory the pass did not create"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:08.474Z
updated_at: 2026-10-01T03:26:32.058Z
started_at: 2026-10-01T00:17:31.818Z
closed_at: 2026-10-01T03:26:32.057Z
close_reason: "Fixed in df829ec8 and 9e53c996: cleanup removes only what the pass made; target-owner before the build; work dir inside a checkout refused. PR #170, disposition map https://github.com/jlevy/fdu/pull/170#issuecomment-5924110534"
resolution: null
duplicate_of: null
---
scripts/release/stability_pass.py:1199-1208 removes a caller's --target-dir and any existing worktree/ or target/ under a reused work directory; the wheel build :771-776 writes to a shared target outside make target-owner. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
