---
type: is
id: is-01m0prgyv1eq0g0mzgntn1p4n6
title: "Partitioned tallies surfaces: --tags and --plane, Selection.plane, per-plane values"
kind: feature
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:54.336Z
updated_at: 2026-08-23T17:01:02.014Z
---
CLI scope axis --tags and selection axis --plane; Python Selection(plane=...), per-plane RollUp/Child/TreeNode values and per-entry tag bits so one children() call serves the dual-value listing; default plane 'all' keeps untagged behavior byte-identical. Goldens with a tagged fixture in every format, replayed by the parity harness.

## Notes

THE SUBTLE PART, and the one easy to get wrong: plane must NOT fall to the slow tier. Selection::is_unfiltered (query_selection.rs:162) is the gate between reading precomputed rollups and the re-aggregating walk (query_report.rs:812). If plane is treated as an ordinary filter it makes every plane query filtered — the 122 ms path the parent spec measured and rejected, against 0.29 ms unfiltered. A plane selects WHICH precomputed rollup to read, so is_unfiltered must stay true for a plane-only query and the section builders (build_section, query_report.rs:916) must route to the plane's rollup. Combining plane with a real filter falls to tier two as any filter does. Report dispatch is report() at query_report.rs:734, which sets walked = None when unfiltered. Goldens need a tagged fixture under tests/golden/fixtures/ with a .gitignore including a negation (tracked in fdu-ey9q).
