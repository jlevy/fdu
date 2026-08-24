---
type: is
id: is-01m0tf9pw281n7hsaenyp0aq3e
title: Bounded tree remainders drop non-file leaves
kind: bug
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels:
  - pr47-review
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T18:07:37.337Z
updated_at: 2026-08-24T18:07:44.071Z
---
At PR 47 head 5012069, TreeNode gains others but Remainder still carries only rows, files, dirs, bytes, and allocated. Remainder::absorb and withheld_children omit others, and the JSON, YAML, and Python remainder shapes cannot carry it. A bounded tree whose omitted directory contains only symlinks says the full node has non-file leaves but cannot account for them in its machine-readable remainder, violating truncate freely never silently and the partition relation. Fix: add others across Remainder, absorb, withheld_children, serializers, and the Python model and conversion. Tests under both limit and depth bounds must assert emitted child others plus remainder.others equals node.others. Review finding FDU47-R9.
