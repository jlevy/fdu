---
type: is
id: is-01m3gafrkpczr2mra09xfvpzge
title: Verify test sensitivity portability and maintenance cost after consolidation
kind: task
status: in_progress
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
delegate: codex-integration
labels: []
dependencies: []
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
created_at: 2026-09-27T02:18:14.005Z
updated_at: 2026-09-27T03:07:40.685Z
---
Verify the revised testing architecture with reviewed golden diffs, portability/observability/invocation guards, the same shared CLI/Python corpus and authoritative Linux parity artifact. Use representative fault injections or deliberately broken behavior to demonstrate key guards fail, not only that the normal suite passes. Record before/after test-code and fixture size plus observed runtime with platform/selection, explaining any necessary growth. Run make check and applicable cross-lint, preserve required coverage, and record residual gaps explicitly before closing the phase.

## Notes

Review findings corrected; 183 shared CLI goldens and 16,787-case full CLI/Python matrix pass. Final local gate and Linux-owned parity recording/CI are running on draft PR133. Fault-injection sensitivity evidence is recorded in the plan-linked correctness artifact.
