---
type: is
id: is-01kzvbc3ga8r0myf3mn9hhybc6
title: Restore compact aligned tree bars and truthful truncation markers
kind: bug
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-12T16:02:34.120Z
updated_at: 2026-08-12T16:15:17.183Z
closed_at: 2026-08-12T16:15:17.182Z
close_reason: Restored fixed 10-cell bars and aligned columns, limited human ellipses to omitted sibling rows, corrected depth-bound truncation semantics, and added Rust/golden/Python-wheel coverage.
---
The five-axis text renderer dropped the prior fixed 10-cell size bars and indents size columns by depth. Its depth cutoff also marks nodes truncated for file children even though the tree view only renders directories, producing spurious ellipses. Restore a compact aligned tree layout and make truncation mean omitted reportable directory rows.
