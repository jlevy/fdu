---
type: is
id: is-01m4gw5755nm0s0q6y3bh3qxw8
title: "PR #192 B3: An earlier root's snapshot save writes into a later root during its walk"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4gw4em1dbpcwbmgqzw911k9
created_at: 2026-10-09T17:42:44.644Z
updated_at: 2026-10-09T21:28:55.703Z
closed_at: 2026-10-09T21:28:55.703Z
close_reason: "fixed on #192 (362ef8f7); dispositions: https://github.com/jlevy/fdu/pull/192#issuecomment-6089558624"
resolution: null
duplicate_of: null
---
Severity: Low. PR #192 review B (https://github.com/jlevy/fdu/pull/192#issuecomment-6085996290), finding B3. Where: execution.rs:936-945; lib.rs:970.
