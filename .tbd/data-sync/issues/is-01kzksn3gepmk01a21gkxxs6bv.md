---
type: is
id: is-01kzksn3gepmk01a21gkxxs6bv
title: Run the CLI golden contract in Make, CI, and review workflow
kind: task
status: in_progress
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies: []
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:38:13.645Z
updated_at: 2026-08-09T18:06:23.524Z
---
Add test-golden and golden-update targets, make the comparison suite part of test/check, run it on Linux/macOS/Windows with pinned Node and locked npm installs, audit npm dependencies, document regeneration and diff review, and prove a stable-output mutation fails.
