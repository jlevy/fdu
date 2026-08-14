---
type: is
id: is-01kzyevafrjwgcfwyy7zmdm3w8
title: Merging a base branch with branch-deletion silently closes every stacked PR
kind: chore
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-13T21:01:01.816Z
updated_at: 2026-08-13T21:01:01.816Z
---
Merging #8 and deleting codex/iterative-performance auto-closed all three PRs stacked on it within four seconds: #10 (content metrics, 10.8k lines), #12 (differential suite), #13 (review follow-ups). None was closed deliberately, and GitHub then refuses both reopen ('state cannot be changed. The <base> branch has been deleted') and retarget ('Cannot change the base branch of a closed pull request'), so each has to be recreated as a new PR. Mitigation for the next stacked merge: retarget dependent PRs to main BEFORE merging their base, or keep the base branch until the stack is drained. Worth a line in the integration runbook so the next stack does not lose a feature PR.
