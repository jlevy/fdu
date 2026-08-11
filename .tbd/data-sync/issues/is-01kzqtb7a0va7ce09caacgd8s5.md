---
type: is
id: is-01kzqtb7a0va7ce09caacgd8s5
title: Automate the runbook's bead-sync check as a periodic guard
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T07:07:16.159Z
updated_at: 2026-08-11T07:07:16.159Z
---
The integration runbook (docs/project/guides/integration-runbook.md) section 8 verifies that a bead's labels, status, priority, and notes survive the round trip through origin/tbd-sync, using scripts/compare_bead_sync.py. It is manual by design, but the comparison itself is mechanical and could run as a scheduled or pre-handoff check over a sample of beads rather than one, catching silent metadata loss without a human remembering to look. Open questions to settle first: which fields form the contract (dependencies and parent edges are not compared today), whether it should fail or only warn, and where it runs so it never blocks a PR on a shared-branch race.
