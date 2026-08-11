---
type: is
id: is-01kzqtb7a0va7ce09caacgd8s5
title: Automate the runbook's bead-sync check as a periodic guard
kind: task
status: open
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T07:07:16.159Z
updated_at: 2026-08-11T21:48:58.594Z
---
The integration runbook (docs/project/guides/integration-runbook.md) section 8 verifies that a bead's labels, status, priority, and notes survive the round trip through origin/tbd-sync, using scripts/compare_bead_sync.py. It is manual by design, but the comparison itself is mechanical and could run as a scheduled or pre-handoff check over a sample of beads rather than one, catching silent metadata loss without a human remembering to look. Open questions to settle first: which fields form the contract (dependencies and parent edges are not compared today), whether it should fail or only warn, and where it runs so it never blocks a PR on a shared-branch race.

## Notes

Partially addressed 2026-08-11 in a1d63f9. The field-contract question is settled: scripts/verify_bead_sync.py compares title, kind, status, priority, spec_path, labels, dependencies, description, and notes - dependencies and notes were precisely what the earlier comparison missed, so the metadata most worth protecting was the metadata least covered. make verify-beads now runs it over every bead (234/234 match) after fetching the sync branch.

Still open, and both hinge on the same fact: the comparison is against origin/tbd-sync, a branch other working copies push to independently. (1) Where a scheduled version runs - not PR CI, where a shared-branch race would fail a pull request for something it did not do. A nightly job against the default branch, or a pre-handoff step, avoids that. (2) Fail or warn - a scheduled job can fail safely; anything in a PR path should warn. Recommend a scheduled workflow that fails, plus the existing manual target for handoffs, but that is a maintainer call about where noise is tolerable.
