---
type: is
id: is-01m2h2vgnr4n0n5gtn922j7xhh
title: "PR #55 review PR55-DUR-1: durable checkpoints lack a compatibility contract"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2h2v87crh401e90pfe52w8g
created_at: 2026-09-14T23:08:23.089Z
updated_at: 2026-09-14T23:54:18.992Z
closed_at: 2026-09-14T23:54:18.990Z
close_reason: "4727de0: checkpoint plan adds Checkpoint Store and Compatibility (separate store, own format version, upgrade rule, per-measure comparability, backward-compatibility block)"
resolution: null
duplicate_of: null
---
PR #55 at dc27c14: checkpoints plan :175-182, :253-255. The cache key mixes crate version, FORMAT_VERSION and CLASSIFICATION_VERSION (origin/main snapshot.rs:173-186) and a scope mismatch cold-scans and replaces the single image (lib.rs:453-471), so a checkpoint keyed like the cache dies on every release. Add a compatibility contract: separate store, own format version, upgrade behaviour, scope and classification recording.
