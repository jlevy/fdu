---
type: is
id: is-01kzx1bkyaej6eprgyw8fk560j
title: "Phase 4c: Lock document metrics with goldens and self-host checks"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1bzbk4an6cccaxe02b9sy
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:46:01.289Z
updated_at: 2026-08-13T10:25:36.787Z
closed_at: 2026-08-13T10:25:36.786Z
close_reason: Document semantics frozen with exact tryscript fixtures, Python parity, cache replay, self-host invariants, and a passing make check on Rust 1.85 plus abi3 Python 3.12.
---
Extend cli-content tryscript, report/2, Python parity, and the multilingual fixture with raw/logical/visible words, paragraphs, page denominators, Markdown links/images/code/tables/HTML/malformed input, and aggregate-before-rounding cases. Extend self-host document invariants; run test-golden, content-selfcheck, and make check before prose optimization.
