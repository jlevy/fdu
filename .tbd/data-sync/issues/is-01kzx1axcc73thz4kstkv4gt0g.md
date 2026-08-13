---
type: is
id: is-01kzx1axcc73thz4kstkv4gt0g
title: "Phase 2a: Implement the fused basic streaming analyzer"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1axs4w29b8sj0t6fx99x4
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:45:38.187Z
updated_at: 2026-08-13T07:45:38.595Z
---
Add content/basic.rs and content/text.rs accumulator primitives described in the spec. Fuse physical/blank/nonblank lines, binary and UTF-8 admission, raw words, paragraph state, and optional logical-word sufficient statistics; pin empty/final-line/LF/CRLF/CR/mixed endings, every chunk boundary, Unicode whitespace, BOM, invalid UTF-8, NUL, and long-line behavior with unit and property tests.
