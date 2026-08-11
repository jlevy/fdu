---
type: is
id: is-01kzsa5ftg7bbvtgmswjda3h5j
title: "PR#6 D4: BFS contract conflates monotonicity with fairness and overpromises ordering"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:59.919Z
updated_at: 2026-08-11T21:02:59.919Z
---
research-2026-08-11-interactive-browser-use-case.md:183-207 and progressive plan. Additive scan is monotone under either order; BFS changes which subtrees fill early. Parallel scheduler is shallow-preferred (see C3). Scope monotonicity to baseline scan over a stable tree. High.
