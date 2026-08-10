---
type: is
id: is-01kzmnxy0xvkvazmqvdwsjm20h
title: Validate and publish the CLI UX follow-up PR
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md
labels:
  - cli
  - pr-review
dependencies: []
parent_id: is-01kzmnx3taexx4cq4m722p0yp0
created_at: 2026-08-10T01:52:23.067Z
updated_at: 2026-08-10T02:17:04.736Z
---
Reconcile the CLI UX plan with the Phase 1 and Rust-quality plans after PR #1 merged, update the native and Python READMEs without claiming unpublished registry availability, run exact goldens, direct local-wheel uvx checks, installed-wheel checks, and make check, review the complete diff, open a dedicated follow-up PR from origin/main with design and remaining later CLI work, push, wait for fresh cross-platform CI, and sync/close completed beads.

## Notes

Final CARGO_INCREMENTAL=0 make check passes on the exact final tree: supply chain, fmt, Clippy, 148 all-feature library tests, four CLI process tests, 26 goldens, docs, 105 core-only, 135 watch-only, exact 1.85, Cargo/npm audits, two Python concurrency tests, installed wheel including native argv, and local-wheel uvx. Remaining: commit/push, open follow-up PR, publish review context, and wait for fresh CI.
