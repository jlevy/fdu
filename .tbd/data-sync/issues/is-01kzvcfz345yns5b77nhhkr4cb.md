---
type: is
id: is-01kzvcfz345yns5b77nhhkr4cb
title: Require an explicit scan path and re-audit composable CLI output
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-12T16:22:09.251Z
updated_at: 2026-08-12T16:47:45.900Z
closed_at: 2026-08-12T16:47:45.899Z
close_reason: Reports now require an explicit PATH; bare fdu is byte-identical to --help, exits 0, and performs no scan. Origin/main comparison confirmed and goldens pin the restored ten-cell bars, structural indentation, and omission markers only for actual sibling limits. README, help, skill, changelog, architecture principles, runbook, and active specs now match the surface. make check passes.
---
Bare fdu must print help and perform no scan; users must provide PATH explicitly. Compare the branch's default human tree rendering with origin/main and correct regressions in bars, alignment, truncation markers, and related UI. Update all affected docs, help, and goldens.
