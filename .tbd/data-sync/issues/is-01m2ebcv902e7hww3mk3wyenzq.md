---
type: is
id: is-01m2ebcv902e7hww3mk3wyenzq
title: "PR #48 review CLASS-3: the v3 registry fingerprint misses keys moving between tiers"
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:56.318Z
updated_at: 2026-09-13T21:39:56.318Z
---
Medium. classify/file_rollup_manifest.rs:405-444. fingerprint() length-prefixes values but writes extensions, filenames, and shebangs back to back with no array boundary, so moving md from extensions to filenames changes classification but not the fingerprint; snapshots and content sidecars recorded under the old registry still match. The compact manifest prefixes array lengths (type_rule_manifest.rs:218-224). Fix: hash each array length and each section count; add a collision test of that shape. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
