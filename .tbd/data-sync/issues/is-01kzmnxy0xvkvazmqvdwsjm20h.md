---
type: is
id: is-01kzmnxy0xvkvazmqvdwsjm20h
title: Validate and publish the CLI UX follow-up PR
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md
labels:
  - cli
  - pr-review
dependencies: []
parent_id: is-01kzmnx3taexx4cq4m722p0yp0
created_at: 2026-08-10T01:52:23.067Z
updated_at: 2026-08-10T02:48:05.218Z
closed_at: 2026-08-10T02:48:05.218Z
close_reason: "PR #2 is open from merged origin/main with the full design and senior review comment. Final CARGO_INCREMENTAL=0 make check passes locally, and fresh PR CI run 31350476952 passes all 13 Linux, macOS, Windows, artifact, MSRV, quality, and supply-chain jobs."
---
Reconcile the CLI UX plan with the Phase 1 and Rust-quality plans after PR #1 merged, update the native and Python READMEs without claiming unpublished registry availability, run exact goldens, direct local-wheel uvx checks, installed-wheel checks, and make check, review the complete diff, open a dedicated follow-up PR from origin/main with design and remaining later CLI work, push, wait for fresh cross-platform CI, and sync/close completed beads.

## Notes

Final CARGO_INCREMENTAL=0 make check passes on commit a4a7e28: supply chain, formatting, strict Clippy, 148 all-feature library tests, four CLI process tests, 26 golden scenarios, docs, 105 core-only, 135 watch-only, exact Rust 1.85, Cargo and npm audits, two Python concurrency tests, installed wheel including native argv, and local-wheel uvx. Follow-up PR #2 is open from merged origin/main with the full design and senior review context. Remaining: wait for fresh Linux, macOS, and Windows CI, resolve any failures, post the final verdict, and close the plan beads.
