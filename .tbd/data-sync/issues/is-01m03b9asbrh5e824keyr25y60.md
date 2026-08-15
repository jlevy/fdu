---
type: is
id: is-01m03b9asbrh5e824keyr25y60
title: Extend scan diagnostics to the FullIndex plan
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - instrumentation
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-15T18:34:58.730Z
updated_at: 2026-08-15T18:34:58.730Z
---
FDU_SCAN_DIAGNOSTICS=1 only emits on the compact Summary plan: prepare_report_internal returns None for RetainedState::FullIndex. The default tree view and every analysis request take FullIndex, so the shipped instrument cannot observe the execution plan users actually run by default.

This matters for field diagnosis. When a user reports that fdu is slow on their tree, the first thing to capture is the policy and backend history of the run they actually made, and today that is exactly the run the trace cannot see.

Acceptance: diagnostics are available on the FullIndex path with the same bounded, versioned fdu-scan-diagnostics-v1 contract and the same opt-in transport; overhead is measured and stays inside the predeclared +3% non-regression margin as exp-056 did for the Summary path; or, if the cost cannot be bounded there, the limitation is documented explicitly in the instrumentation playbook rather than left implicit.
