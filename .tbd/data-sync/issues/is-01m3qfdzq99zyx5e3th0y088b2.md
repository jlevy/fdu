---
type: is
id: is-01m3qfdzq99zyx5e3th0y088b2
title: Make the folded tree index a FoldedIndex newtype instead of five debug asserts
kind: task
status: open
priority: 4
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3qdjym66ky971ft6yjdq7s8
created_at: 2026-09-29T20:59:19.656Z
updated_at: 2026-09-29T20:59:19.656Z
---
Review suggestion S1 on #161: five debug_assert!(!index.is_folded()) guards plus a pub(crate) constructor keep a folded (H172 transient) index from escaping. A FoldedIndex newtype returned by scan_into_folded_index and accepted by one report_folded entry point would make that a type-level fact. Optional; the current guards are adequate and tested.
