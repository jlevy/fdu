---
type: is
id: is-01m31fbmpgajer03wejhrby3qm
title: make check supply-chain scan walks nested .claude/worktrees and fails on untracked hooks
kind: bug
status: open
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-09-21T07:54:45.326Z
updated_at: 2026-09-21T07:57:03.756Z
---
`make check` fails at the `supply-chain` target on any clone that has a git worktree under `.claude/worktrees/`, which is where Claude Code places worktrees by default. Observed 2026-09-21:

```
supply-chain: .claude/worktrees/agent-a0cd895219f35eb23/.claude/hooks/tbd-closing-reminder.sh
  uses a download or package-execution primitive but is absent from the bootstrap inventory
make: *** [supply-chain] Error 1
```

`scripts/check-supply-chain.mjs:693` walks `[".claude", ".codex", "scripts"]` with `shellFilesUnder(root, directory)`, which recurses into nested worktrees. Those files are untracked from the outer repo's point of view (`git ls-files --error-unmatch` fails on them), and they belong to a different checkout, so the bootstrap inventory can never list them.

Impact: the handoff gate is unrunnable locally while any agent worktree exists in the default location, and the failure message points at a hook file rather than at the real cause, which reads like a genuine supply-chain violation. This host had 17 such worktrees.

Fix: have the scanner consider only files the repository actually tracks, or skip any directory containing a `.git` file/dir (a nested worktree or submodule). Prefer `git ls-files` over a filesystem walk so the scanned set matches what is committed. Add a regression test that creates a nested worktree with a hook using a download primitive and asserts the check still passes.

Note for whoever fixes it: do not "fix" this by adding the nested paths to the inventory, and do not delete other agents' worktrees to make the gate pass — several on this host hold uncommitted work.

## Notes

Context posted for Linux handoff on PR #94: https://github.com/jlevy/fdu/pull/94#issuecomment-5757234985
