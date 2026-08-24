---
type: is
id: is-01m0tfbt0djry86tn8bd03ydr9
title: Replace the stale implementation bead count with the live graph
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels:
  - pr47-review
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T18:08:46.091Z
updated_at: 2026-08-24T19:22:02.381Z
closed_at: 2026-08-24T19:22:02.380Z
close_reason: |
  Fixed. The sentence was stale as it landed, exactly as the finding says -- `fdu-u7vo`
  expands through implementation and review, and this review added children while the
  sentence was being written.

  The implementation spec's "What Landed" section now states no count at all. It points at
  `tbd list --parent fdu-u7vo --all` as the live map and says what the document actually is:
  the tables below are the shipped rows, each naming a bead and where its work landed, and
  those rows are the claim. A number that has to be maintained by hand in a file nobody
  re-reads is a number that will be wrong.

  The PR body carries the same defect and is edited separately -- it is not version
  controlled, so it is not part of this change.

  `make check` green.
resolution: null
duplicate_of: null
---
PR 47 commit 5012069 changes the implementation narrative to Twenty-three of the twenty-six beads under the contract epic are closed. The live fdu-u7vo map has expanded substantially through active scope and review findings, so this copied numerator and denominator are already false and cannot stay maintained. Replace the snapshot count with the command that reports current state, tbd list --parent fdu-u7vo --all, or scope the prose to the explicit shipped rows in the adjacent table without claiming an epic total. Review finding FDU47-R10.
