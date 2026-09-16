---
type: is
id: is-01m2m1ncwwfw6sk3ytrngg8avh
title: "Unpushed review-fix checkpoints for #63, #65 and #67"
kind: task
status: open
priority: 1
version: 2
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-16T02:45:17.337Z
updated_at: 2026-09-16T03:15:51.715Z
---
Three review-fix streams were stopped mid-flight by an API session limit at about 19:35 on 2026-09-15. Their work is checkpointed locally so a lost worktree does not lose it.

- #63 (`claude/control-bounds-degrade`): six commits on branch `w-63` in worktree `agent-a3a179c9f23e29611`, tree clean, covering the two-limit split (`--gitignore-budget`, `--gitignore-line-limit`) and every finding of review 5215446267. The coordinator is running `make check` on it before pushing.
- #67 (`claude/cache-clear-old-formats`): WIP commit `c336fa4` in worktree `agent-a3cdfe1233424b7c8`, unverified, covering PR67-1 through PR67-6 of review 5216601111 (the `fdu.cache/1` schema string, leftover-file classification, docs, JSON shape, the vacuous test).
- #65 (`claude/gitignore-default-on`): WIP commit `ae75073` in worktree `agent-aba5795aa9c93db57`, unverified, covering F1 (watch repaints when a `.gitignore` edit moves an entry into or out of an ignored-state selection) and F2 (the raw `query::report()` path refuses rather than answering zero rows) of review 5217139212.

Each PR body carries a status section saying what is pushed, what is reviewed, and what is unverified and unpushed. Resume the agents after the session limit resets (20:40 America/Los_Angeles), or redo the work from the review reports in `~/.cache/fdu-release-work/reviews/`.

## Notes

Handoff for landing 0.1.0 (ordered steps, ground rules, open user decisions, known traps): ~/.cache/fdu-release-work/HANDOFF.md, written 2026-09-15 20:15 PT.
