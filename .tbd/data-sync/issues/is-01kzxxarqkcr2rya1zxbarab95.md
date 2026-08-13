---
type: is
id: is-01kzxxarqkcr2rya1zxbarab95
title: Apply the behavior-preserving Rust module rename
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-13-rust-module-filenames.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzxxas41qe9r26g86hwxvmds
parent_id: is-01kzxxaqvy1gjmb4vdcg40tcr3
created_at: 2026-08-13T15:54:53.553Z
updated_at: 2026-08-13T16:09:48.107Z
closed_at: 2026-08-13T16:09:48.103Z
close_reason: Applied the approved Git-aware module and integration-test rename map, retained the fdu::session compatibility alias, changed no behavior or golden expectation, and passed the complete make check gate.
---
From a green golden baseline, use Git-aware moves and exact repren substitutions to apply the approved rename map. Preserve public item paths and fdu::session compatibility, change no function bodies or expected output, and validate Rust, Python, and CLI golden behavior before a dedicated structural commit.
