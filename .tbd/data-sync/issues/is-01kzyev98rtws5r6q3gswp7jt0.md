---
type: is
id: is-01kzyev98rtws5r6q3gswp7jt0
title: "Retarget the stranded content-metrics work onto the PR #14 branch"
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-13T21:01:00.568Z
updated_at: 2026-08-13T21:01:00.568Z
---
Follow-up to fdu-dn4u, requested during the post-#8 audit. Once PR #14 (claude/pr-8-senior-review-egv3mq) is ready, re-land the content-metrics work on top of that branch rather than against main directly. GitHub cannot reopen or retarget closed PR #10 (its base branch is deleted), so: branch from codex/file-content-metrics-plan at fbb36f8, merge the #14 branch into it to move it off the deleted pre-merge base, resolve conflicts against the #8 stabilization (macos_bulk H1 fix, permission_bits_are_enforced fixtures, attribution removal, parallel_equivalence), run the full gate, and open a new PR with base claude/pr-8-senior-review-egv3mq. Preserve the 40-commit history and its resolved review threads.
