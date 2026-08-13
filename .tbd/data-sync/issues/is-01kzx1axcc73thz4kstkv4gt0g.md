---
type: is
id: is-01kzx1axcc73thz4kstkv4gt0g
title: "Phase 2a: Implement the fused basic streaming analyzer"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1axs4w29b8sj0t6fx99x4
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:45:38.187Z
updated_at: 2026-08-13T08:33:16.750Z
closed_at: 2026-08-13T08:33:16.744Z
close_reason: Implemented content-basic-v1 as a fused arbitrary-chunk accumulator with NUL and UTF-8 admission, BOM handling, LF/CRLF/lone-CR/mixed boundaries, Unicode blank lines, physical/blank/nonblank identities, raw words, paragraph runs, and additive FlexDoc-style logical statistics. Every two- and three-chunk split of a multibyte/mixed-ending fixture matches one-shot output; early/late NUL and invalid UTF-8 discard provisional metrics. Clippy and focused all-feature tests pass.
---
Add content/basic.rs and content/text.rs accumulator primitives described in the spec. Fuse physical/blank/nonblank lines, binary and UTF-8 admission, raw words, paragraph state, and optional logical-word sufficient statistics; pin empty/final-line/LF/CRLF/CR/mixed endings, every chunk boundary, Unicode whitespace, BOM, invalid UTF-8, NUL, and long-line behavior with unit and property tests.
