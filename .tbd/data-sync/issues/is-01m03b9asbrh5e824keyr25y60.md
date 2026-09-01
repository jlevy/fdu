---
type: is
id: is-01m03b9asbrh5e824keyr25y60
title: Extend scan diagnostics to the FullIndex plan
kind: task
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
delegate: codex@spud10.local
labels:
  - performance
  - instrumentation
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
hold: null
hold_until: null
created_at: 2026-08-15T18:34:58.730Z
updated_at: 2026-09-01T15:01:21.944Z
started_at: 2026-09-01T14:38:40.994Z
closed_at: 2026-09-01T15:01:21.943Z
close_reason: Cold FullIndex scans now expose the bounded fdu-scan-diagnostics-v1 trace through the existing opt-in API and installed-CLI transport; cache-only and warm-reconcile limitations are explicit and tested. Exp-090 measured diagnostics-on at -3.48% with paired 95% CI [-11.88%, +1.43%], below the +3% ceiling, with exact tallies and all resource gates held. Rust all-feature/no-default tests, clippy, the 221-test performance harness, schemas, ledger, report, and docs checks pass.
resolution: null
duplicate_of: null
---
FDU_SCAN_DIAGNOSTICS=1 only emits on the compact Summary plan: prepare_report_internal returns None for RetainedState::FullIndex. The default tree view and every analysis request take FullIndex, so the shipped instrument cannot observe the execution plan users actually run by default.

This matters for field diagnosis. When a user reports that fdu is slow on their tree, the first thing to capture is the policy and backend history of the run they actually made, and today that is exactly the run the trace cannot see.

Acceptance: diagnostics are available on the FullIndex path with the same bounded, versioned fdu-scan-diagnostics-v1 contract and the same opt-in transport; overhead is measured and stays inside the predeclared +3% non-regression margin as exp-056 did for the Summary path; or, if the cost cannot be bounded there, the limitation is documented explicitly in the instrumentation playbook rather than left implicit.
