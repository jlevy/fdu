---
type: is
id: is-01kzy3tj319v3g93s9ee0zdhgm
title: "PR #8 senior review: root-safe fixtures, group-kill probe, and first Linux measurements"
kind: task
status: closed
priority: 2
version: 2
labels:
  - perf
  - linux
dependencies: []
created_at: 2026-08-13T17:48:22.496Z
updated_at: 2026-08-13T17:48:29.563Z
closed_at: 2026-08-13T17:48:29.563Z
close_reason: Delivered on branch claude/pr-8-senior-review-egv3mq; PR to follow stacked on codex/iterative-performance
---
Deliverables of the PR #8 senior engineering review, stacked on codex/iterative-performance: (1) three permission-drop fixtures now probe that access was actually revoked and skip under CAP_DAC_OVERRIDE, keeping make check green in root containers while staying fully armed unprivileged; (2) the benchmark harness group-cleanup test treats an unreaped zombie as dead on Linux, fixing a false failure under lazy-reaping inits; (3) benchmarks/spikes/ adds the single-file walker (seven enumeration/stat strategies incl. hand-rolled io_uring statx), the adjacent-paired runner, and the deterministic tree generator; (4) docs/project/research/research-2026-08-13-linux-first-measurements.md records the first Linux cross-tool numbers (syscall convergence, exposed index consumer, warm-open inversion, mimalloc/io_uring/inosort spike verdicts). Follow-up experiments live in fdu-maxn, fdu-niuz, fdu-91ts, fdu-cckr, fdu-tk1b, fdu-i2f3, fdu-dzs0, fdu-lf3v. Review comment: https://github.com/jlevy/fdu/pull/8#issuecomment-5284260287
