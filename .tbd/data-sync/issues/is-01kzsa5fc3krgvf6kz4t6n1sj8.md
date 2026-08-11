---
type: is
id: is-01kzsa5fc3krgvf6kz4t6n1sj8
title: "PR#6 D2: exp-012 supports 'no detected wall regression', not 'free' or 'unchanged in memory'"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:59.458Z
updated_at: 2026-08-11T21:02:59.458Z
---
exp-012 artifact reports RSS +1.51% [+0.85,+2.88] cold-scan-index, CPU +2.50% and RSS +3.66% cold-scan-producer, CPU +2.31% warm-revalidate. Docs claim 11 MB unchanged; artifact medians are 33-36 MiB. Also 'significant' = ci95_high<0 renders regressions as n.s. High.
