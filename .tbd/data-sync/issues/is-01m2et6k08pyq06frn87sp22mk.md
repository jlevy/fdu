---
type: is
id: is-01m2et6k08pyq06frn87sp22mk
title: "PR #48 review FIX48-1: restored Complete coverage is contradicted one level down"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2et5s9yyv9e6tz783rp2m6c
created_at: 2026-09-14T01:58:39.879Z
updated_at: 2026-09-14T02:49:12.013Z
closed_at: 2026-09-14T02:49:11.995Z
close_reason: "Fixed in b803b8e (option 1): a complete opened-root reconcile records the completeness of the directories it listed; LIFE-3 test asserts lookup-below Absent; tests for directories created after discovery via refresh and observer. CI 19/19 at 40ecf28. https://github.com/jlevy/fdu/pull/48#issuecomment-5658305981"
resolution: null
duplicate_of: null
---
Medium, introduced by c801d4e (widens fdu-k18s). At f917cb7: index.rs:1636-1643 (Watching re-derives Complete), read.rs:873-893 absence_is_known, read.rs:53-59. Discovery commits 'blocked' as a directory, fails to read it, root is Partial(Inaccessible); 'blocked' becomes readable before the handoff; the handoff's full pass reads it without error and Watching restores Complete, but a reconcile never marks children complete, so Lookup blocked/missing answers Unknown { Building } forever on a Watching/Complete root. Chosen fix (option 1): fix fdu-k18s -- a complete, error-free reconcile records directory completeness through the commit path, as discovery's DirectoryComplete does; add the lookup-below assertion to watching_after_a_clean_handoff_rederives_complete_coverage and a test for a directory created after discovery. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5193206420
