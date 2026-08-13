---
type: is
id: is-01kzxxaqvy1gjmb4vdcg40tcr3
title: Clarify and enforce Rust module filenames
kind: epic
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-13-rust-module-filenames.md
labels: []
dependencies: []
child_order_hints:
  - is-01kzxxarav6fd7375sjtrwfam2
  - is-01kzxxarqkcr2rya1zxbarab95
  - is-01kzxxas41qe9r26g86hwxvmds
  - is-01kzxxasgg35kcqrfx5f03n3aq
created_at: 2026-08-13T15:54:52.669Z
updated_at: 2026-08-13T15:54:54.351Z
---
Deliver a safe stacked refactor that gives ambiguous Rust source files self-describing, repository-unique basenames, removes production mod.rs files, preserves external behavior and compatibility, adds a narrow automated guardrail, and proves the result through unchanged CLI goldens and the full handoff gate.
