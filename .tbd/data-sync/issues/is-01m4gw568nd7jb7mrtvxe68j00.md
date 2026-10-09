---
type: is
id: is-01m4gw568nd7jb7mrtvxe68j00
title: "PR #192 B1: With one root, a file PATH fails as I/O before delivery refusals"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4gw4em1dbpcwbmgqzw911k9
created_at: 2026-10-09T17:42:43.732Z
updated_at: 2026-10-09T21:28:55.684Z
closed_at: 2026-10-09T21:28:55.681Z
close_reason: "fixed on #192 (362ef8f7); dispositions: https://github.com/jlevy/fdu/pull/192#issuecomment-6089558624"
resolution: null
duplicate_of: null
---
Severity: Low. PR #192 review B (https://github.com/jlevy/fdu/pull/192#issuecomment-6085996290), finding B1. Where: crates/fdu/src/cli.rs:860-882; crates/fdu-py/src/lib.rs:1077.
