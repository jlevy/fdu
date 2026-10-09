---
type: is
id: is-01m4gw5d6emv8tvstrhvxp26t0
title: "PR #192 C7: Several-root text output allocates a joined path for every row"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4gw4em1dbpcwbmgqzw911k9
created_at: 2026-10-09T17:42:50.828Z
updated_at: 2026-10-09T21:28:58.212Z
closed_at: 2026-10-09T21:28:58.209Z
close_reason: "declined: one label-join rule (labelled_path) over a second prefix-writing text path; one root borrows as before; dispositions: https://github.com/jlevy/fdu/pull/192#issuecomment-6089558882"
resolution: null
duplicate_of: null
---
Severity: Low. PR #192 review C (https://github.com/jlevy/fdu/pull/192#issuecomment-6085911373), finding C7. Where: report_format.rs:341.
