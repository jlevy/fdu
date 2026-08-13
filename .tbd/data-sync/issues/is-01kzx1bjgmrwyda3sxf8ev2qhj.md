---
type: is
id: is-01kzx1bjgmrwyda3sxf8ev2qhj
title: "Phase 3d: Run evidence-gated SLOC performance iterations"
kind: task
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1bk3smf8r9eabkqtsazvn
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:45:59.804Z
updated_at: 2026-08-13T12:03:03.808Z
closed_at: 2026-08-13T10:03:42.188Z
close_reason: Added cold/cache-hit SLOC evidence jobs and metric-complete semantic digests, measured SCC 3.7.0/Tokei 14.0.0 on generated and immutable corpora, profiled code analysis, recorded/reverted rejected H66, and retained Python 3.12 support.
---
Only after the SLOC semantic lock, add code-sloc-cold and code-sloc-cache-hit jobs plus pinned SCC/Tokei comparators on generated and immutable self-host corpora. Profile and preregister iterations, preserve semantic digests, use the paired acceptance protocol, and record both accepted and rejected experiments.
