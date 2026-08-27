---
type: is
id: is-01m10nrazfqj0ndxdpvv94kprg
title: Replace exact-remainder page assembly with bounded continuation safety
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nrbaqfz5va594e01y265j
parent_id: is-01m0y1sgqd1sd33stssgw25f2q
created_at: 2026-08-27T03:55:54.734Z
updated_at: 2026-08-27T07:56:14.197Z
closed_at: 2026-08-27T07:56:14.196Z
close_reason: "Completed in MetaBrowser commit 45266a8: tree and catalog assembly follow opaque version-pinned continuations with independent page and row bounds."
resolution: null
duplicate_of: null
---
Update tree_page_assembly.py assemble_tree_pages/TreePageAssembly and coordinator read_session/_compose_read_locked. Enforce stable version, positive bounds, advancing unique continuations, maximum pages/rows/work, and provider order; request product totals separately in the same coherent read and never resort, rescan, or reconstruct them.
