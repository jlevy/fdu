---
type: is
id: is-01kzqts3m1asn1bhjr6tehgqn5
title: Lock and CI-test the performance evidence toolchain
kind: bug
status: closed
priority: 2
version: 2
labels:
  - pr-review
dependencies: []
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T07:14:51.136Z
updated_at: 2026-08-11T07:21:32.182Z
closed_at: 2026-08-11T07:21:32.181Z
close_reason: Added a cool-off-compliant locked uv project for softschema/Pydantic, made ISO dates portable and validation fail closed, covered both uv locks in provenance checks, added 70 evidence tests to make check and cross-platform CI, and passed the full gate.
---
Additional final audit finding: the realtree evidence suite imports Pydantic but make perf-test uses an undeclared environment, softschema is invoked through an unpinned runner, and CI does not run the 68 evidence-contract tests. Pin a cool-off-compliant softschema release in a frozen uv project, make ISO dates YAML-portable, and run the suite in CI.
