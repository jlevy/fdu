---
type: is
id: is-01kzky7fjvk5f7758cav879nhs
title: Reject malformed observation batches before any index mutation
kind: bug
status: open
priority: 0
version: 9
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - correctness
  - api
  - merge-blocker
dependencies:
  - type: blocks
    target: is-01kzky7pe77x2wqndf6kdwyn6p
  - type: blocks
    target: is-01kzky7wjz44trprn1ck52pd58
  - type: blocks
    target: is-01kzky86nqp91wq9d3wj2psnwr
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T18:58:10.138Z
updated_at: 2026-08-09T21:07:50.803Z
---
Index::apply currently treats an absolute path or a path containing parent traversal as unchanged, even though Error::PathEscapesRoot exists and public Observation values can be constructed by external producers. Add red tests for absolute, parent, prefix, and mixed valid-invalid batches. Validate the complete batch at one boundary and return a typed error before changing entries, freshness, pending invalidations, the journal, or the clock. Update direct, shared, scan, watch, snapshot, CLI, and Python callers so malformed producer input can never look like an ordinary no-op. Mark the resulting operation outcome as must-use where Result does not already enforce it.
