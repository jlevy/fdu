---
type: is
id: is-01m0prgyv1eq0g0mzgntn1p4n6
title: "Partitioned tallies surfaces: --tags and --plane, Selection.plane, per-plane values"
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:54.336Z
updated_at: 2026-08-23T07:33:04.312Z
---
CLI scope axis --tags and selection axis --plane; Python Selection(plane=...), per-plane RollUp/Child/TreeNode values and per-entry tag bits so one children() call serves the dual-value listing; default plane 'all' keeps untagged behavior byte-identical. Goldens with a tagged fixture in every format, replayed by the parity harness.
