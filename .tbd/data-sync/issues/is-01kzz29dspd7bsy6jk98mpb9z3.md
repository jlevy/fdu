---
type: is
id: is-01kzz29dspd7bsy6jk98mpb9z3
title: "Salvage the still-valid fixes from PR #4 and retire the rest"
kind: epic
status: open
priority: 1
version: 12
labels:
  - performance
  - correctness
dependencies: []
child_order_hints:
  - is-01kzz29wnq90e1md2dx4zspdhb
  - is-01kzz29x4thpjssb3keq3yk2at
  - is-01kzz29xkn8rqb5p7367jpwb2h
  - is-01kzz2aka6tqk2hr1284zvn3se
  - is-01kzz2akt6txq8epjvdgcx0n5s
  - is-01kzz2amb356zx5q0yj0h41ays
  - is-01kzz2amst9cgp4sn6xtc0x0sz
  - is-01kzz2bak4xbxszt2na2yhmeq0
  - is-01kzz2bb2fzm1ctw3w5y90w451
  - is-01kzz2bbj7wbktyst23qwx2c8t
  - is-01kzz31zpz1y045pvjf6k77ccw
created_at: 2026-08-14T02:40:46.901Z
updated_at: 2026-08-14T02:54:11.679Z
---
PR #4 (codex/address-pr3-review, head 8a13373) branched from e7e2e08 on 2026-08-10 and never merged. main has since advanced through PRs #5, #8, #9, #14, #15, #16, and #18, which renumbered the experiment ledger to exp-050, replaced the cross-environment matrix approach with the platform-tuning guide plus the spike harness, and rebuilt much of the scan and index hot path.

Most of PR #4 is therefore obsolete: the 110k lines of archived exp-000..exp-012 evidence, the env-001 environment matrix (environment.py, decision.py, scale.py, environment-matrix.schema.yaml, performance-environment.yml), and the plan-doc edits for the composable CLI surface that shipped in PR #5.

A three-way diff of base vs PR #4 vs main shows a set of real defects that main still carries because they were only ever fixed on the abandoned branch. This epic tracks porting those, adapted to main's current code, and retiring PR #4.
