---
type: is
id: is-01m3kfjdk3rpq2w1927xnbqssp
title: Merge upward and link new layers into stack 141; keep PR stack tables current
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
parent_id: is-01m3kfj7vjnh8patz58z2p2bh1
created_at: 2026-09-28T07:44:47.200Z
updated_at: 2026-09-28T07:45:07.124Z
---
For each new layer (docs, macOS performance, watch fix, 0.2.0 release): merge the layer below into it (never rebase), push, gh stack link 141 <branch>, then update the stack table in every stack PR body (#137, #138, #139, #142, #143 and new PRs) with branch@head rows and the merge instruction count. User approved the lead doing these merges. Mark PRs ready for review when their content is final (gh stack merge refuses drafts; #143 is a draft owned by another session — ask the user before changing it).
