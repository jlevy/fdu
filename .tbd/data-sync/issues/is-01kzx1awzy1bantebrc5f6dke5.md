---
type: is
id: is-01kzx1awzy1bantebrc5f6dke5
title: "Phase 1: Land stable classification and metric contracts"
kind: task
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1axcc73thz4kstkv4gt0g
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:45:37.790Z
updated_at: 2026-08-13T12:03:02.017Z
closed_at: 2026-08-13T08:30:27.213Z
close_reason: Added versioned analyzer/slot/profile/coverage/provenance contracts, semantic option fingerprints, additive metrics, sparse ContentIndex records, precomputed directory/type/family rollups, owned candidates, and one conditional apply boundary. Metadata updates and subtree removals subtract content immediately; stale worker results are rejected. Disabled profiles create no candidates and Index retains content=None. Clippy, all-feature tests, rustdoc, and docs gates pass.
---
After fdu-v4lc, update classify.rs with stable type/family/provenance contracts; add content/types.rs and the sparse disabled ContentIndex boundary; extend query/report.rs with the generic metadata metric projection and extensions/types/families views. Pin metadata-only Rust, Python, CLI, report/1, snapshot-v2, allocation, and performance compatibility. Use only the isolated feature worktree.
