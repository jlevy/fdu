---
type: is
id: is-01m4hahwke1809m4k6ktcabcx2
title: "PR #192 D2: Docs recommend fdu */ without exp-216's cost or the symlinked-sibling refusal"
kind: bug
status: in_progress
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4hahez6scpwtc6yrg9qws5e
hold: null
hold_until: null
created_at: 2026-10-09T21:54:19.875Z
updated_at: 2026-10-09T21:54:31.585Z
started_at: 2026-10-09T21:54:31.582Z
---
Medium. docs/usage.md:91-92,:106; CHANGELOG.md:39-42; crates/fdu/src/cli.rs:196 (--docs), :1086-1089 (refusal remedy); crates/fdu/src/skills/SKILL.md:102-103. State per-walk cost (625 roots 1.72 s vs 0.23 s, ~7x, macOS, uncontrolled), one-walk alternative, lib64 -> lib refusal. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
