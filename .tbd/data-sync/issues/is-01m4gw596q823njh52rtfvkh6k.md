---
type: is
id: is-01m4gw596q823njh52rtfvkh6k
title: "PR #192 C1: Refactored single-root paths have no paired measurement"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4gw4em1dbpcwbmgqzw911k9
created_at: 2026-10-09T17:42:46.742Z
updated_at: 2026-10-09T21:28:56.961Z
closed_at: 2026-10-09T21:28:56.960Z
close_reason: "fixed on #192 (b7f72218..fa62a7b1); dispositions: https://github.com/jlevy/fdu/pull/192#issuecomment-6089558882"
resolution: null
duplicate_of: null
---
Severity: High. PR #192 review C (https://github.com/jlevy/fdu/pull/192#issuecomment-6085911373), finding C1. Where: query_report.rs:1983, 1822; execution.rs:660, 763; report_format.rs TextTree.
