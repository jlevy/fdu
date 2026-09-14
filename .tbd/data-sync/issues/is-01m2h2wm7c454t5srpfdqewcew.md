---
type: is
id: is-01m2h2wm7c454t5srpfdqewcew
title: "PR #58 review PR58-REC-3: exp-104 verdict.commit holds the control hash"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m2h2t7k65srszyhv02tag8ba
created_at: 2026-09-14T23:08:59.488Z
updated_at: 2026-09-14T23:27:40.290Z
closed_at: 2026-09-14T23:27:40.289Z
close_reason: "bd8ed0b: verdict.commit set to null (schema allows it); ledger header and timeline.json regenerated."
resolution: null
duplicate_of: null
---
PR #58, P3. exp-104 `verdict.commit` is `dda7e6af`, which is `method.control`.

Runbook RECORD: `--commit` names the commit that contains the change. A rejected, never-committed candidate records `null` (exp-098, exp-100, exp-103). The schema allows `null` (`docs/project/experiments/experiment.schema.yaml:519-523`).

Fix: set `null`, then `make perf-ledger` and `make perf-report`.
