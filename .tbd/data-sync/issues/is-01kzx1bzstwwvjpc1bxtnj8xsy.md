---
type: is
id: is-01kzx1bzstwwvjpc1bxtnj8xsy
title: "Phase 5: Add bounded deep detection and specialized formats"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1c089k0ssb8t3vy000fq9
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:46:13.433Z
updated_at: 2026-08-13T07:46:13.896Z
---
Add content/detect.rs shebang, required-literal, ambiguity, modeline, XML/manpage, generated/vendor, and specialized-format rules only behind bounded probes and named consumers. Keep resolved extensions on the constant-time path, expose provenance/confidence and coverage, add ambiguity-maximizing goldens, and measure detect-ambiguous before expanding coverage.
