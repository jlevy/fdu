---
type: is
id: is-01kzx1bkh01q3gvbwpcpxyrfk9
title: "Phase 4b: Implement reader-visible Markdown prose metrics"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1bkyaej6eprgyw8fk560j
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:46:00.863Z
updated_at: 2026-08-13T07:46:01.289Z
---
Prototype and gate pulldown-cmark, then add content/markdown.rs to fold parser events directly into visible raw/logical word statistics and paragraph blocks. Retain reader-visible labels and opted-in table cells; exclude destinations, definitions, code, frontmatter, footnote markers, and hidden markup. Bound buffers and report too-large coverage.
