---
type: is
id: is-01kzsjhx2sgx22h3e9c9gcrp7b
title: Reconcile sweep does not use the region scheduler; breadth-first costs +2.70% there
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-11T23:29:35.320Z
updated_at: 2026-08-11T23:29:35.320Z
---
Measured in exp-014 (20 interleaved paired trials, same binary both arms): warm-revalidate wall +2.70% [+1.55%, +3.37%], component +4.50% [+2.68%, +5.15%], cpu +2.48% [+1.18%, +3.17%] for breadth-first vs depth-first. Cause: reconcile/revalidate walk with the serial take_next over one VecDeque, so region scheduling (exp-013) never reached them and breadth-first there is still the front-popping global FIFO, paying locality and frontier costs. On the warm sweep a one-shot CLI reads none of the orientation benefit, since it prints only after reconciliation completes. Two options, neither measured: (a) extend region scheduling to the reconcile sweep, (b) let the sweep default to DepthFirst and take BreadthFirst only from a caller that reads progressively -- closer to the project position that traversal order is a consumer contract. Gate either through the accept rule on warm-revalidate.
