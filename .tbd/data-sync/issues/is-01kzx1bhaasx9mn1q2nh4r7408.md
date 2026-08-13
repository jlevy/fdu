---
type: is
id: is-01kzx1bhaasx9mn1q2nh4r7408
title: "Phase 3b: Implement code-sloc-v1 and the languages report"
kind: task
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1bhwb9vp0hvm365w7kaxw
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:45:58.601Z
updated_at: 2026-08-13T12:03:03.414Z
closed_at: 2026-08-13T09:45:26.698Z
close_reason: Integrated the dependency-free streaming code-sloc-v1 analyzer for 15 common languages, explicit unsupported coverage, sidecar v2 metrics, Rust/CLI/Python projections, and code-line-based language shares.
---
Implement the selected fdu-owned-buffer code analyzer for the required common languages; define code/comment/blank/physical, mixed-line, string, docstring, nesting, generated, and embedded-language rules; report unsupported coverage; and expose the languages preset with code-line and optional byte shares across Rust, CLI, report/2, and Python.
