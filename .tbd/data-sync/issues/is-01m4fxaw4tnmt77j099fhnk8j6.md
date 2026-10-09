---
type: is
id: is-01m4fxaw4tnmt77j099fhnk8j6
title: "Roots: run each root's plan and read all indexes into one report"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies:
  - type: blocks
    target: is-01m4fxax3s3274rk55njmnrksk
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T08:44:04.121Z
updated_at: 2026-10-09T09:06:03.540Z
---
Execution: each root's plan in argument order retains summary row / folded index / full index for one read; per-root pending saves returned and completed by the caller; one cumulative progress handle naming the current root and position; any failing root fails the run; --allow-partial makes the report partial. Reader: accumulate/finalize split per section (single root = one input). Summary sums (ignored share unknown if any root's is); grouped views merge buckets and pooled word stats, observed = AND; code adds; flat views classify and bound per root, then existing sort_rows+truncate over the concatenation, total sums. Tree: section total + one tree per root; expand() takes the combined denominator; roots always rows (zero-byte too), no breadth on roots, depth per root, roots ordered by the existing sorter with labels as names; --limit and --limit 0 once over the assembled pre-order with per-root completeness; per-root remainders. Machine: envelope root null + roots[{label,label_raw?,path,path_raw?}]; tree null + total + trees; rows, errors, refusals gain root index with root-relative paths. Status/provenance/notes merges per the spec.
