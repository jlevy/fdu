---
type: is
id: is-01m2h2vn038gdgcxdne7mp4gks
title: "PR #55 review PR55-ACCT-1: signed allocated-byte ranking overcounts hard links and clones"
kind: bug
status: in_progress
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2h2v87crh401e90pfe52w8g
created_at: 2026-09-14T23:08:27.473Z
updated_at: 2026-09-14T23:09:12.653Z
---
PR #55 at dc27c14: checkpoints plan :69-74, :96-97, :116-119, :272-275. Default ranking by signed allocated bytes counts hard links and APFS clones in full on package caches, the first target subjects. Engine retains inode and dev but no link count (origin/main engine_contract.rs:81-95). Define delta accounting; the related engine policy is fdu-579b.
