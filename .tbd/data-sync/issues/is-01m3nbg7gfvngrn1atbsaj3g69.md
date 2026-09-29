---
type: is
id: is-01m3nbg7gfvngrn1atbsaj3g69
title: "Caveat H140/H146 leftover determinations: most of the linux-v6.12 default command was .gitignore classification, not the syscall floor"
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-29T01:12:09.999Z
updated_at: 2026-09-29T01:12:09.999Z
---
exp-139 (H140) attributed ~96% of the linux-v6.12 default tree to the syscall floor and exp-147 (H146) likewise; exp-173 measured 590 ms with .gitignore vs 82 ms without, so most was consumer-thread classification inside the walk-phase timer. Caveat the registry rows and the runbook's Linux Standing; consider a new determination with a .gitignore-off arm.
