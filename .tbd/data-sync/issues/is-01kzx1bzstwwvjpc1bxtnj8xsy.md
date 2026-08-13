---
type: is
id: is-01kzx1bzstwwvjpc1bxtnj8xsy
title: "Phase 5: Add bounded deep detection and specialized formats"
kind: task
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1c089k0ssb8t3vy000fq9
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:46:13.433Z
updated_at: 2026-08-13T12:03:04.798Z
closed_at: 2026-08-13T10:57:56.669Z
close_reason: Implemented bounded deep detection, report/cache/Python evidence, ambiguity and binary goldens, and validated isolated performance jobs.
---
Add content/detect.rs shebang, required-literal, ambiguity, modeline, XML/manpage, generated/vendor, and specialized-format rules only behind bounded probes and named consumers. Keep resolved extensions on the constant-time path, expose provenance/confidence and coverage, add ambiguity-maximizing goldens, and measure detect-ambiguous before expanding coverage.
