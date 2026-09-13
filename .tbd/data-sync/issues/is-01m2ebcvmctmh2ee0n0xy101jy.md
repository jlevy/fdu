---
type: is
id: is-01m2ebcvmctmh2ee0n0xy101jy
title: "PR #48 review CLASS-4: an unknown extension has two canonical answers"
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:56.683Z
updated_at: 2026-09-13T21:39:56.683Z
---
Medium. classify.rs:559-574 vs 593-597. For release.v2.widget, TypeRegistry::canonical_ext and ext_bucket answer .widget (used by per-directory extension tallies, index.rs:3286) while classify_name sets canonical_extension only when a rule matched, so the portable row (index.rs:1965) says None. Fix: check File Rollup Format's rule for unknown kinds, make one method delegate to the other, add a conformance case for an unclaimed compound extension. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
