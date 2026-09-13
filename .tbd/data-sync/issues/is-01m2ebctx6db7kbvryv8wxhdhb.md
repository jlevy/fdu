---
type: is
id: is-01m2ebctx6db7kbvryv8wxhdhb
title: "PR #48 review CLASS-2: derive_ext no longer gives main's answer"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:55.941Z
updated_at: 2026-09-13T23:38:36.360Z
closed_at: 2026-09-13T23:38:36.359Z
close_reason: "Fixed in 3b6de62, option 1: derive_ext is main's raw rule; File Rollup eligibility applies only to logical_ext and declared matching; tallies, extension view, and unknown labels use the raw extension; no CLI golden moved. CI green."
resolution: null
duplicate_of: null
---
Medium. classify.rs:878-885, 949-983. derive_ext became canonical_ext(logical_ext(name)) with the File Rollup eligibility rule (ASCII alphanumeric, at most 12 units), so file.c++, notes.tar.gz~, a.b-c, x.py_, x.abcdefghijklm, résumé.tëxt move to (none); the doc comment and plan :1860 say the public helper's answer is preserved. Fix (pick one): keep derive_ext on main's permissive rule and apply eligibility only in logical_ext/classify_name, or accept and document with a golden fixture. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
