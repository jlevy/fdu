---
type: is
id: is-01kzxxaqvy1gjmb4vdcg40tcr3
title: Clarify and enforce Rust module filenames
kind: epic
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/done/plan-2026-08-13-rust-module-filenames.md
labels: []
dependencies: []
child_order_hints:
  - is-01kzxxarav6fd7375sjtrwfam2
  - is-01kzxxarqkcr2rya1zxbarab95
  - is-01kzxxas41qe9r26g86hwxvmds
  - is-01kzxxasgg35kcqrfx5f03n3aq
created_at: 2026-08-13T15:54:52.669Z
updated_at: 2026-08-13T16:27:08.696Z
closed_at: 2026-08-13T16:27:08.696Z
close_reason: "Published stacked PR #11 after unchanged 92-case goldens, full local make check, multilingual repository smoke validation, and a green macOS/Linux/Windows CI matrix."
---
Deliver a safe stacked refactor that gives ambiguous Rust source files self-describing, repository-unique basenames, removes production mod.rs files, preserves external behavior and compatibility, adds a narrow automated guardrail, and proves the result through unchanged CLI goldens and the full handoff gate.
