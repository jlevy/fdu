---
type: is
id: is-01kzmnxy0xvkvazmqvdwsjm20h
title: Validate and publish the CLI UX follow-up PR
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md
labels:
  - cli
  - pr-review
dependencies: []
parent_id: is-01kzmnx3taexx4cq4m722p0yp0
created_at: 2026-08-10T01:52:23.067Z
updated_at: 2026-08-10T02:09:06.780Z
---
Reconcile the CLI UX plan with the Phase 1 and Rust-quality plans after PR #1 merged, update the native and Python READMEs without claiming unpublished registry availability, run exact goldens, direct local-wheel uvx checks, installed-wheel checks, and make check, review the complete diff, open a dedicated follow-up PR from origin/main with design and remaining later CLI work, push, wait for fresh cross-platform CI, and sync/close completed beads.

## Notes

Local implementation review and make check are green. Remaining: finalize docs/diff, sync tbd, commit and push codex/cli-ux-agent-skill, open the dedicated follow-up PR from origin/main, publish the senior review context, and wait for fresh CI across the expanded native/Python matrices.
