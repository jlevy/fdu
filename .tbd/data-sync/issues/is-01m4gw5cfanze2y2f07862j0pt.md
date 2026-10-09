---
type: is
id: is-01m4gw5cfanze2y2f07862j0pt
title: "PR #192 C6: With --cache on, save threads are unbounded in the number of roots"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4gw4em1dbpcwbmgqzw911k9
created_at: 2026-10-09T17:42:50.084Z
updated_at: 2026-10-09T21:28:57.018Z
closed_at: 2026-10-09T21:28:57.018Z
close_reason: "fixed on #192 (b7f72218..fa62a7b1); dispositions: https://github.com/jlevy/fdu/pull/192#issuecomment-6089558882"
resolution: null
duplicate_of: null
---
Severity: Low. PR #192 review C (https://github.com/jlevy/fdu/pull/192#issuecomment-6085911373), finding C6. Where: execution.rs:944; snapshot.rs:1210.
