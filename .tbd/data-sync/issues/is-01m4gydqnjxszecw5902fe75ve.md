---
type: is
id: is-01m4gydqnjxszecw5902fe75ve
title: Model a report's roots and a tree's rows as enums
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T18:22:20.849Z
updated_at: 2026-10-09T18:22:20.849Z
---
PR #192 review A10 (https://github.com/jlevy/fdu/pull/192#issuecomment-6085921156): `Report { root: Option<PathBuf>, roots: Option<Vec<NamedRoot>> }` and `Section::Tree { root: Option<Box<TreeNode>>, roots: Option<Box<RootTrees>>, .. }` encode 'exactly one of' as two Options, so a consumer matching `Section::Tree { root: Some(..), .. }` silently skips several-root trees as if the row limit were zero. Proposed: `enum ReportRoots { One(PathBuf), Several(Vec<NamedRoot>) }` and `enum TreeRows { One(Option<Box<TreeNode>>), Several(Box<RootTrees>) }` (or One | Several | Omitted), with today's wire shape kept by the serializer and the Python models unchanged. Deferred from #192 because it reshapes a field that predates the PR (`Section::Tree::root`) at about 50 match sites across fdu-core, the command line, the Python binding's conversions, perf_probe, and the watch and opened-root readers, and is best done with the one-shot API consolidation (one `prepare(&RootsRequest, &Delivery, Options)`, review A's what-next 4) so 0.x breaks the report API once rather than twice. Must land before 1.0.
