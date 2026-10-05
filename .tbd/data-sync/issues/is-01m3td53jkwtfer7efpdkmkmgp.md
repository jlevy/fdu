---
type: is
id: is-01m3td53jkwtfer7efpdkmkmgp
title: "PR #170 review R10: the pty probe's forked child can run the parent's code"
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
created_at: 2026-10-01T00:17:14.823Z
updated_at: 2026-10-01T03:26:33.470Z
started_at: 2026-10-01T00:17:35.563Z
closed_at: 2026-10-01T03:26:33.469Z
close_reason: "Fixed in a2e5d541: the probe's child always exits. PR #170, disposition map https://github.com/jlevy/fdu/pull/170#issuecomment-5924110534"
resolution: null
duplicate_of: null
---
scripts/qa/pty_probe.py:112-127: only os.execve is inside try/finally os._exit(127); a raise in setsid, ioctl, dup2 or os.open unwinds the child through probe() and main(), whose finally deletes the shared cache directory. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
