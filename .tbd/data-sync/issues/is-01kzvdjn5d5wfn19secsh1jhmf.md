---
type: is
id: is-01kzvdjn5d5wfn19secsh1jhmf
title: Keep regenerated performance ledger Flowmark-clean
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-12T16:41:05.964Z
updated_at: 2026-08-12T16:47:46.108Z
closed_at: 2026-08-12T16:47:46.107Z
close_reason: make perf-ledger now runs the repository-standard pinned flowmark --auto . pass after generation. Regenerating all 15 experiment entries followed by make docs-format-check leaves the ledger unchanged and clean.
---
PR #5 review found that make perf-ledger writes the generated ledger before the repository-wide Flowmark normalizer runs, so the documented generator can leave output that fails make docs-format-check. Route regeneration through the same pinned flowmark --auto . standard and prove regeneration plus the format check is clean.
