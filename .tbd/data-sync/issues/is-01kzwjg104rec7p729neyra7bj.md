---
type: is
id: is-01kzwjg104rec7p729neyra7bj
title: Make canonical live-tree baseline and measurement contiguous
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
dependencies: []
parent_id: is-01kzwewqvqrt0sa5h4j4p3bmpa
created_at: 2026-08-13T03:26:17.091Z
updated_at: 2026-08-13T05:51:08.101Z
closed_at: 2026-08-13T05:51:08.100Z
close_reason: The v2 tool harness now fingerprints immediately before and after timing, writes evidence outside the subject, requires one immutable FDU binary, and produced a clean zero-drift definitive run.
---
The first 1M-entry cross-tool run was internally stable but invalidated because git status refreshed .git/index between the external baseline and the harness pre-fingerprint. Make the canonical target regenerate its redacted baseline immediately before measurement, keep both outputs outside the subject, test the safety boundary, and document that no git/tbd/build command may overlap the run.

## Notes

Implemented immediate pre-run fingerprint plus external --baseline-output; canonical Make target no longer depends on a separately generated baseline and sets PYTHONDONTWRITEBYTECODE=1. Smoke run: zero invalid samples, no baseline drift, identical embedded before/after digest, output outside subject.
