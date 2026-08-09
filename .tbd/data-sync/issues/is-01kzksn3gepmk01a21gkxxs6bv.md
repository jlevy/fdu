---
type: is
id: is-01kzksn3gepmk01a21gkxxs6bv
title: Run the CLI golden contract in Make, CI, and review workflow
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies: []
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:38:13.645Z
updated_at: 2026-08-09T18:39:22.495Z
closed_at: 2026-08-09T18:39:04.091Z
close_reason: The 25-scenario CLI golden contract is wired into Make and locked audits, its update workflow is proven, and CI run 31329423861 passes it on Linux, macOS, and Windows.
---
Add test-golden and golden-update targets, make the comparison suite part of test/check, run it on Linux/macOS/Windows with pinned Node and locked npm installs, audit npm dependencies, document regeneration and diff review, and prove a stable-output mutation fails.
