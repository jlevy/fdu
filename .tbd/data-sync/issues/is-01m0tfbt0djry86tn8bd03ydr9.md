---
type: is
id: is-01m0tfbt0djry86tn8bd03ydr9
title: Replace the stale implementation bead count with the live graph
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels:
  - pr47-review
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T18:08:46.091Z
updated_at: 2026-08-24T18:08:46.091Z
---
PR 47 commit 5012069 changes the implementation narrative to Twenty-three of the twenty-six beads under the contract epic are closed. The live fdu-u7vo map has expanded substantially through active scope and review findings, so this copied numerator and denominator are already false and cannot stay maintained. Replace the snapshot count with the command that reports current state, tbd list --parent fdu-u7vo --all, or scope the prose to the explicit shipped rows in the adjacent table without claiming an epic total. Review finding FDU47-R10.
