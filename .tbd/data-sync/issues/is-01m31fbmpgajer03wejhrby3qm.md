---
type: is
id: is-01m31fbmpgajer03wejhrby3qm
title: make check supply-chain scan walks nested .claude/worktrees and fails on untracked hooks
kind: bug
status: closed
priority: 1
version: 7
labels: []
dependencies: []
parent_id: is-01m31hc3p7wv76jeq5dhgv3bd8
created_at: 2026-09-21T07:54:45.326Z
updated_at: 2026-09-21T17:08:20.833Z
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

Severity evidence: because the gate stops at supply-chain, two real defects in PR #103 reached CI undetected (stale Python format contract on all platforms, and a Windows-only path-separator test failure). Any agent on a host with a Claude Code worktree under .claude/worktrees/ has no working local gate.

FIXED 2026-09-21 on `claude/amazing-mayer-2ps368`, commit `5daab1d6`.

`shellFilesUnder` now reads `git ls-files --cached` instead of walking the filesystem. The index is both the correct set to audit — the policy governs what is committed, and a fresh checkout is exactly the tracked files — and the one set immune to whatever else is on disk. `--cached` includes staged entries, so a new downloader is still caught before it lands.

Reproduced before and after rather than assumed. Against the old walk the fixture yields two un-inventoriable paths (the nested `.claude/hooks/tbd-closing-reminder.sh` and a nested copy of the tracked script); against the new code it yields only `scripts/reviewed.sh`.

The scan is not weakened into a no-op. This repository tracks five shell files under the scanned directories and all five use a download primitive, so all five must be inventoried; the gate still verifies every one and reports "70 Cargo packages, 60 npm packages, 25 Python packages, 57 action uses, and all bootstrap pins".

The regression test builds a real nested worktree rather than a fake `.git` entry, because the bug was git's own notion of tracking rather than a name. Side effect worth knowing: a missing directory is now empty rather than an error, so `.codex` no longer has to exist for the scan to run.

Confirmed on a Linux host with no nested worktree: `make check` now proceeds past `supply-chain` — the first time the gate has run since this bug was found.
