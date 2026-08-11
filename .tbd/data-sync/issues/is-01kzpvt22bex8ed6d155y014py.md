---
type: is
id: is-01kzpvt22bex8ed6d155y014py
title: Profile and optimize compatible-snapshot real-tree revalidation
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzpvt29hvtsrg1pyrq20awxa
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-10T22:13:36.458Z
updated_at: 2026-08-11T00:24:47.936Z
---
Using the same immutable real-tree subject, profile compatible-unchanged open/revalidation, snapshot load, first listing, full completion, and verified-warm repetitions. Preserve the rule that directory fingerprints may skip only name-set discovery, never child truth checks. Use exact pre/post oracles, interleaved paired trials, resource evidence, and one commit per accepted optimization; reject complexity without stable end-to-end gains.

## Notes

Warm path partly addressed and the bottleneck located. exp-002 rejected and reverted: parallelising the revalidation sweep gained only 2.6%, proving the warm path is bound by the single index consumer rather than by traversal. exp-004 accepted (borrowed path components): warm-revalidate -9.4%, snapshot-load -17.8%. exp-005 accepted (snapshot load resolves through the parent rather than from the root): snapshot-load -18.6%. Cumulative warm-revalidate 804->688 ms (-14.7%), warm-snapshot-load 324->230 ms (-29.1%). OUTSTANDING DEFECT: the cached path is still slower than no cache at all (CLI measured 753 ms cached against 167 ms uncached), so the headline feature remains a pessimization. The structural fix is the snapshot format work in H10.
