---
type: is
id: is-01m0racc8tf20x27jjhh35vh5q
title: "Bundled coherent read: one guard, one version, cursor, state, fingerprints"
kind: feature
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T22:03:13.050Z
updated_at: 2026-08-23T22:43:00.438Z
closed_at: 2026-08-23T22:43:00.438Z
close_reason: IndexHandle::read(&ReadRequest) -> ReadBundle evaluates children, named roll-ups, and whole-tree totals under one read guard, returning the clock every part saw (and the cursor to resume from), the scan scope with its ignore/type-rule/reducer fingerprints, freshness, entry count and root. Python Index.read(...) returns a typed Bundle carrying that plus the run's completeness, source, and typed errors. The coherence test runs a concurrent writer and asserts the rows sum to the header; the negative was verified by hand — replacing the bundle with two separate calls fails within a few iterations (6 against 4) — so it passes because the guard holds, not because nothing overlapped. Child rows are built by one shared function in Rust and parsed by one in Python, so a bundle cannot describe a child differently from children().
resolution: null
duplicate_of: null
---
Several projections evaluated under one read guard, returning one engine version, the change cursor captured at the same boundary, index state (lifecycle, coverage, freshness, source, progress, typed issues), and the scope and registry fingerprints. A composed response must not straddle a commit, and a consumer cache key must derive from what was actually read rather than a revision sampled before dispatch. This is the primitive metabrowser's Phase 2 opening spike exercises -- open a shared handle, one bundled directory-plus-rollup read, one version/cursor/state/work record -- so it lands early with fdu-gav9 rather than drifting late. Also collapses per-call FFI overhead.
