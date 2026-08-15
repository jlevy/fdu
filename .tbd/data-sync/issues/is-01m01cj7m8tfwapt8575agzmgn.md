---
type: is
id: is-01m01cj7m8tfwapt8575agzmgn
title: "PR #26 review S2: dispatch release rehearsal to prove full matrix"
kind: task
status: open
priority: 2
version: 3
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
created_at: 2026-08-15T00:18:50.120Z
updated_at: 2026-08-15T00:31:51.742Z
---
Deferred: after R1/R11 land on the PR and merge, dispatch release.yml once so all five wheel legs, evidence, and registry classification actually run before fdu-9cf0.

## Notes

Dispatch is impossible until the workflow reaches main: GitHub only registers workflow_dispatch workflows present on the default branch (verified 2026-08-15 via the Actions workflow list; only ci.yml and performance-environment.yml are registered). Runner-label and deployment-target fixes for the matrix landed in 862190a on claude/fdu-pr-review-g8rsrm. After merge: dispatch release.yml once and confirm all five wheel legs, the evidence job, and registry classification pass before starting fdu-9cf0.
