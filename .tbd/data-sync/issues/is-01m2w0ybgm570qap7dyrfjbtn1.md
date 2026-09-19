---
type: is
id: is-01m2w0ybgm570qap7dyrfjbtn1
title: Agent-ready performance loop handoff
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/guides/performance-loop-runbook.md
labels:
  - macos-agenda
dependencies: []
parent_id: is-01m0py28fhtj20wwfc05s9e148
created_at: 2026-09-19T05:06:37.715Z
updated_at: 2026-09-19T05:08:48.741Z
closed_at: 2026-09-19T05:08:48.740Z
close_reason: "Runbook Current Standing (2026-09-18) is the pickup: one-iteration commands, uncontrolled rustup baseline, H107-H111 next-up, dead ends, process pack. Loop guide, campaign-2 plan, campaign status, and AGENTS.md point at it. Landed on PR #91 as bd03cd6c."
resolution: null
duplicate_of: null
---
Close only when a stranger agent can run the next performance-loop iteration from docs alone (no chat).

Required in-repo, one source of truth (extend the runbook / loop guide / campaign-2 plan; do not add a parallel doc):

1. How to run one iteration: worktree/branch, quiet vs uncontrolled, Darwin subjects after the 2026-09-18 re-nomination, make perf-compare / perf-record / perf-ledger / perf-report, accept-rule-before-measure, when to revert.
2. Current standing best and regime (exp-105 uncontrolled rustup baseline). README 200K files/s and 4M cached lines/s stay; do not raise from probe data. Installed-CLI QA 2026-09-18 is a different table.
3. Next-up list for H107–H111 with metric, subject, why open, what falsifies, what not to retry.
4. Process pack pointers: ledger, evidence report, campaign-2, instrumentation (FDU_COUNTERS=1), First Principles, engine architecture.

Single branch perf/campaign-quiet-2026-09-18, PR #91. Do not merge. Do not force-push.
