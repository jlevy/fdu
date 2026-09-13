---
type: is
id: is-01m2ebctx6db7kbvryv8wxhdhb
title: "PR #48 review CLASS-2: derive_ext no longer gives main's answer"
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:55.941Z
updated_at: 2026-09-13T21:39:55.941Z
---
Medium. classify.rs:878-885, 949-983. derive_ext became canonical_ext(logical_ext(name)) with the File Rollup eligibility rule (ASCII alphanumeric, at most 12 units), so file.c++, notes.tar.gz~, a.b-c, x.py_, x.abcdefghijklm, résumé.tëxt move to (none); the doc comment and plan :1860 say the public helper's answer is preserved. Fix (pick one): keep derive_ext on main's permissive rule and apply eligibility only in logical_ext/classify_name, or accept and document with a golden fixture. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
