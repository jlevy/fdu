---
type: is
id: is-01m0y1sc1h17y99grptjb9pzha
title: Route every existing producer through exact commits
kind: task
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sce6j2y5ac1nzgtsdwsx
  - type: blocks
    target: is-01m0y1se38tcc11akkz34mjrme
  - type: blocks
    target: is-01m0y1sed8zrkf6hdnp5wrq5ty
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:28.208Z
updated_at: 2026-08-26T03:28:56.484Z
---
Route cold scan, reconcile, pending reconcile, explicit refresh, control updates, and watch verification through Index::commit_prepared. Remove requested-observation delta reconstruction and guessed live ancestry; verify unknown ancestry from the nearest known ancestor and preserve current one-shot answers.
