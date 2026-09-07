---
type: is
id: is-01m1x5t3acttkxkwfybncp9ssb
title: Verify each revision in cross-revision performance provenance
kind: bug
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - validation
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T05:35:13.739Z
updated_at: 2026-09-07T06:09:36.510Z
---
Final parity preparation found that provenance.capture assigns one source_root revision to every artifact, and the measurement verifier checks only that same checkout. With tags_at_commit present, _fdu_revision_reasons rejects a legitimate old-control/new-candidate pair because the control version cannot match the candidate HEAD. Held-out comparisons therefore cannot currently establish honest cross-revision source provenance through the supported CLI. Reproduce with tests first, then add explicit per-artifact source checkout attribution and verification, preserve path redaction, reject dirty/mismatched/missing control sources, and retain current single-source callers. Do not relax claim-grade validation or fabricate artifact origins. Use the fixed contract for the final historical and immediate-control comparisons.

## Notes

Full integrated make check passed, including 224 realtree tests, every Rust feature gate, docs, artifact validation, Python package and CLI parity, and release tests. Provenance fix committed and pushed; final CI pending.
