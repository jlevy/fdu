---
type: is
id: is-01m2pmrmvr6x2kz662jksackef
title: "JSONL writer rewrites strings: collapse() edits bracketed paths"
kind: bug
status: open
priority: 0
version: 1
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - output
  - release
dependencies: []
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
created_at: 2026-09-17T02:57:35.607Z
updated_at: 2026-09-17T02:57:35.607Z
---
report_format.rs collapse() joins pretty JSON lines and replaces "{ ", " }", "[ ", " ]" globally, so a path
`a [ b/f { g }.txt` is emitted as `a [b/f {g}.txt` (confirmed on the release build). Also: --watch --format yaml
emits JSON change records. Resolved by writers serializing the answer model directly.
