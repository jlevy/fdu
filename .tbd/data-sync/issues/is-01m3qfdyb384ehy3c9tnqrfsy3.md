---
type: is
id: is-01m3qfdyb384ehy3c9tnqrfsy3
title: "Perf schema: a verdict for non-regression screens so shipped macOS screens are not counted as failed"
kind: task
status: open
priority: 4
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3qdjym66ky971ft6yjdq7s8
created_at: 2026-09-29T20:59:18.243Z
updated_at: 2026-09-29T20:59:18.243Z
---
Review suggestion on #158: exp-164, exp-165 and exp-167 were pre-registered as non-regression screens but the schema can only record them as rejected with kept: candidate, so the evidence page's lede and per-platform table count them among the failed experiments. Add a decision value (e.g. screen) or a pre_registered_bar field, and regenerate the ledger and report. Only worth doing if the failed count is quoted.
