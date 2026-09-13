---
type: is
id: is-01m2ebdp32y4vq1m3w2q02qbz5
title: "PR #48 review CLASS-7: the registry parser's TOML subset is unverified against the real registry"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:23.777Z
updated_at: 2026-09-13T21:40:23.777Z
---
Low. classify/file_rollup_manifest.rs:66-77, 226-243. The hand-written parser rejects valid TOML forms (BOM, inline comments, multi-line arrays, literal strings) and keeps backslash-quote escapes literally in labels; rejection is total and typed, but it was never checked against the actual shared registry document. Fix: add a fixture of the real registry file. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
