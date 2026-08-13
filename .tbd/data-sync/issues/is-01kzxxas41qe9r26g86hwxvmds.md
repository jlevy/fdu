---
type: is
id: is-01kzxxas41qe9r26g86hwxvmds
title: Enforce the Rust filename policy
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-13-rust-module-filenames.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzxxasgg35kcqrfx5f03n3aq
parent_id: is-01kzxxaqvy1gjmb4vdcg40tcr3
created_at: 2026-08-13T15:54:53.952Z
updated_at: 2026-08-13T15:54:54.351Z
---
Add and test a dependency-free repository checker for production mod.rs files, forbidden catch-all basenames, and duplicate non-root Rust basenames. Integrate it into make check in a commit separate from the structural rename.
