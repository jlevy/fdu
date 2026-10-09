---
type: is
id: is-01m4fyjg32yetj9va263jqezc2
title: Give remainder rows an age from a maximum kept in the folded tally
kind: feature
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T09:05:42.497Z
updated_at: 2026-10-09T09:05:42.497Z
---
0.5.0 leaves the remainder row's age cell blank because the folded one-shot tier counts most files in a tally without their times. Keeping a max mtime in the folded tally (one comparison per folded file) and in RowFacts/TreeOmission would let both routes give '… and N more files' the newest activity among hidden rows.
