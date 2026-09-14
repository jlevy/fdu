---
type: is
id: is-01m2eesfetv82m7h4scmj3se2j
title: Directories added after discovery are never marked complete
kind: bug
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-09-13T22:39:15.928Z
updated_at: 2026-09-14T01:49:45.618Z
---
Found while addressing PR #48 review LIFE-3 (not a finding of that review). Every directory entry inserted by an upsert starts with children_complete = false (crates/fdu-core/src/index.rs upsert_beneath: children_complete: kind != EntryKind::Dir), and only discovery's DiscoveryCommit.directory_complete ever sets it true (index.rs apply_opened_state). So a directory a refresh, the observation handoff, or the live observer adds -- even after walking it completely -- stays incomplete for the session: a Lookup below it answers Unknown { reason: Building } on a root whose coverage is Complete and whose phase is Watching, which never resolves (opened/read.rs absence_is_known and coverage_reason). Coverage::Complete is documented as 'every directory in scope has a complete child listing' (engine_contract.rs), which these entries contradict. Direction to decide: have a complete reconcile of a subtree record directory completeness through the commit path (with its DirectoryComplete transitions and progress count), or narrow the Coverage contract to discovery's scope and give such lookups a truthful reason.
