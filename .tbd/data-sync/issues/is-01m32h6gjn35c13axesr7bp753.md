---
type: is
id: is-01m32h6gjn35c13axesr7bp753
title: "PR #108 review pass: close two gates that pass the regressions they catch"
kind: task
status: in_progress
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m32h5emeefrv7vx5yrx70fga
created_at: 2026-09-21T17:46:08.853Z
updated_at: 2026-09-21T18:01:43.771Z
---
Branch `claude/gate-integrity`, based on main. Reviewer model: Fable.

State to reach, both as comments on the PR:
1. Senior review per `tbd shortcut review-github-pr`, in the artifact format (scope, verdict, numbered findings with severities and a concrete Fix, suggestions, false positives, CI status).
2. Per-finding disposition per `tbd shortcut address-pr-review` — each finding fixed, rebutted, or explicitly deferred.

Merged up to `main` at c7babf76 on 2026-09-21 before review, so the diff reviewed is the one that would merge.

Review scope is this layer only; a finding about lower-layer code belongs on the lower PR.

## Notes

Senior review posted: https://github.com/jlevy/fdu/pull/108#issuecomment-5765055008 (verdict MERGE WITH CHANGES, 9 findings, 3 blocking).
Disposition posted: https://github.com/jlevy/fdu/pull/108#issuecomment-5765134606 — all nine addressed in 07a3cd42.

The blocking one was in my own runbook and was the exact class it exists to catch: the mechanism check guarded cache-only on `only_rc == 0`, and since the engine either serves cache_only or exits 1, a build that never wrote a snapshot exited 1 and skipped the check — 17 of 23 cases printed ok against a dead cache. Now every clause is reachable and freshness is asserted. Verified against a wrapper forcing --cache off: 29 mechanism failures, exit 1, while answer mismatches stay 0 in both directions.

Also fixed: sameSeparator missing from the new equality rule (a telemetry-only session with a path became unexplained); a personal absolute path in both scripts; a per-class test passing for the wrong reason on 3 of 4 classes; a corpus test that tested nothing; tests/correctness outside PYTHON_LINT_PATHS and failing ruff; build_tree.py misreporting six kinds on rerun; an over-claim about macOS staleness (fdu-hb2t reopened for the Windows measurement); and the runbook's missing cadence and inbound link.
