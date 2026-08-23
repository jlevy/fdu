---
type: is
id: is-01m0raccjvpde63hx884rkmq5d
title: Scalar paged child rows with remainder, no extension-map copy
kind: feature
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T22:03:13.370Z
updated_at: 2026-08-23T22:03:13.370Z
---
children() clones every child and each directory child's complete by_ext map (verified: IndexHandle::children builds an owned RollUp per directory child over an unbounded BTreeMap). Split the listing from the breakdown: child rows carry scalar directory facts, classification identity, tags, and provenance, with an explicit row bound, a page cursor, and a stated remainder. The extension breakdown becomes its own bounded rollup projection requested only for the directory being inspected. Closes the spec's open question 'does children() need its own bound' as yes. Work proportional to visible output, never one FFI call per child nor an unbounded clone.
