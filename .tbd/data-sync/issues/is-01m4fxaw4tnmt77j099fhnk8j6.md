---
type: is
id: is-01m4fxaw4tnmt77j099fhnk8j6
title: "Roots: run each root's plan and read all indexes into one report"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m4fxax3s3274rk55njmnrksk
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-09T08:44:04.121Z
updated_at: 2026-10-09T16:17:20.329Z
started_at: 2026-10-09T16:17:19.999Z
closed_at: 2026-10-09T16:17:20.328Z
close_reason: Reader through per-root state (e55d0a94); report shape, merges, tree assembly, machine and text output for several roots (625c7966, with WIP b653446a); execution retaining per-root state, pending saves, one progress handle, combined outcome (bb0dc086, with WIP 1b1e3ae3)
resolution: null
duplicate_of: null
---
Execution: each root's plan in argument order retains summary row / folded index / full index for one read; per-root pending saves returned and completed by the caller; one cumulative progress handle naming the current root and position; any failing root fails the run; --allow-partial makes the report partial. Reader: accumulate/finalize split per section (single root = one input). Summary sums (ignored share unknown if any root's is); grouped views merge buckets and pooled word stats, observed = AND; code adds; flat views classify and bound per root, then existing sort_rows+truncate over the concatenation, total sums. Tree: section total + one tree per root; expand() takes the combined denominator; roots always rows (zero-byte too), no breadth on roots, depth per root, roots ordered by the existing sorter with labels as names; --limit and --limit 0 once over the assembled pre-order with per-root completeness; per-root remainders. Machine: envelope root null + roots[{label,label_raw?,path,path_raw?}]; tree null + total + trees; rows, errors, refusals gain root index with root-relative paths. Status/provenance/notes merges per the spec.
