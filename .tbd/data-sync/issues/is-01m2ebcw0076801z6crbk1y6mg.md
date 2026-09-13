---
type: is
id: is-01m2ebcw0076801z6crbk1y6mg
title: "PR #48 review READ-3: Tree and RollUp contradict Lookup for a present file"
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:57.055Z
updated_at: 2026-09-13T21:39:57.055Z
---
Medium. opened/read.rs:420-423, 63-79, 847-867. On a complete root, Tree on a file path answers Knowledge::Absent (whose contract, engine_contract.rs:748, is proven absence) and RollUp answers Unknown{Building}, which never resolves, while Lookup answers Present. Fix: one typed present-but-not-a-directory result in both projections; never Absent, never Unknown for a retained path. Envelope defect under fdu-91ru. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
