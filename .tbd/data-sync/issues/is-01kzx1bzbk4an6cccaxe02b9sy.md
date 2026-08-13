---
type: is
id: is-01kzx1bzbk4an6cccaxe02b9sy
title: "Phase 4d: Run evidence-gated text and Markdown performance iterations"
kind: task
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1bzstwwvjpc1bxtnj8xsy
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:46:12.978Z
updated_at: 2026-08-13T12:03:04.600Z
closed_at: 2026-08-13T10:40:53.578Z
close_reason: Completed semantic locks, self-host and prose workloads, cache-path benchmarks, rejected H67, accepted H68, and passed the full make check gate including cp312-abi3 wheel validation.
---
Only after the document semantic lock, add text-prose and markdown-prose jobs covering ordinary English, long tokens and URLs, punctuation, CJK, links, code fences, tables, HTML, malformed markup, cache hits, and immutable fdu self-host data. Profile, preregister, preserve digests, and use the paired acceptance protocol.
