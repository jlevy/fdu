---
type: is
id: is-01m0tdy9ceep2byvbtyvwc2vky
title: Release the GIL and measure the full Python read boundary
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels:
  - pr47-review
  - performance
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:54.508Z
updated_at: 2026-08-24T17:44:16.257Z
---
At PR 47 head e658915, PyIndex.read calls IndexHandle::read while holding the GIL. As fdu-samw adds filtered tree, recent, and catalog projections, native O(entries) work on an asyncio worker thread will still freeze the event-loop thread because the extension owns the GIL. The Work record also omits CPU time and binding-copy bytes, and wall_ns stops before Python object construction, despite closed fdu-qgl9 promising those dimensions. Fix: detach the native read and reacquire the GIL only for bounded conversion; count bytes materialized across that conversion. Measure CPU exactly in an opt-in detailed mode or mark it unavailable in both contracts rather than deriving it from wall minus lock wait. Test the actual Index.read path for concurrent Python progress and include conversion in the performance harness. Review finding FDU47-R6.
