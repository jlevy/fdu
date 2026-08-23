---
type: is
id: is-01m0racc8tf20x27jjhh35vh5q
title: "Bundled coherent read: one guard, one version, cursor, state, fingerprints"
kind: feature
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T22:03:13.050Z
updated_at: 2026-08-23T22:03:13.050Z
---
Several projections evaluated under one read guard, returning one engine version, the change cursor captured at the same boundary, index state (lifecycle, coverage, freshness, source, progress, typed issues), and the scope and registry fingerprints. A composed response must not straddle a commit, and a consumer cache key must derive from what was actually read rather than a revision sampled before dispatch. This is the primitive metabrowser's Phase 2 opening spike exercises -- open a shared handle, one bundled directory-plus-rollup read, one version/cursor/state/work record -- so it lands early with fdu-gav9 rather than drifting late. Also collapses per-call FFI overhead.
