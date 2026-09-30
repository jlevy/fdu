---
type: is
id: is-01kzsjhx2sgx22h3e9c9gcrp7b
title: Reconcile sweep does not use the region scheduler; breadth-first costs +2.70% there
kind: bug
status: open
priority: 3
version: 3
labels: []
dependencies: []
created_at: 2026-08-11T23:29:35.320Z
updated_at: 2026-09-30T10:02:09.197Z
---
Measured in exp-014 (20 interleaved paired trials, same binary both arms): warm-revalidate wall +2.70% [+1.55%, +3.37%], component +4.50% [+2.68%, +5.15%], cpu +2.48% [+1.18%, +3.17%] for breadth-first vs depth-first. Cause: reconcile/revalidate walk with the serial take_next over one VecDeque, so region scheduling (exp-013) never reached them and breadth-first there is still the front-popping global FIFO, paying locality and frontier costs. On the warm sweep a one-shot CLI reads none of the orientation benefit, since it prints only after reconciliation completes. Two options, neither measured: (a) extend region scheduling to the reconcile sweep, (b) let the sweep default to DepthFirst and take BreadthFirst only from a caller that reads progressively -- closer to the project position that traversal order is a consumer contract. Gate either through the accept rule on warm-revalidate.

## Notes

Reviewed with jlevy: 2.70% is below the project's own 3% bar for changes worth added complexity, and the queue ahead of it is worth far more (fdu-tt2j ~2x on cold large trees; fdu-1vd0 turns an 11s warm load into a first paint; open is still ~28% of cold self-time). Documented as an accepted cost in exp-014, the browser research, and the progressive-results plan rather than left as outstanding work. Revisit only if the warm path becomes the bottleneck.

2026-09-30 stability pass (claude/stability-fixes, read at 42ef6c23): the premise is half stale. reconcile_direct_parallel, the parallel reconcile wave, seeds a DirectoryQueueState, which is the region scheduler exp-013 introduced, so the parallel sweep is region-scheduled. The serial sweeps still take from one VecDeque through take_next(order): the serial walk, revalidate, and the serial reconcile, where BreadthFirst is the front-popping global FIFO exp-014 measured, and BreadthFirst is the default order. The options are as before: (a) region-schedule the serial sweeps, a scheduler change on the warm path, or (b) let a serial sweep default to DepthFirst unless the caller reads progressively, a one-line policy change that is still a measured-path change: it needs a paired warm-revalidate cell under the accept rule and a ledger entry before it can be kept, and no cell was run tonight. Not changed in this pass: below the 3% bar, and a traversal-order change without its cell is not small and safe. Revisit with a cell if the warm path becomes the bottleneck, as the notes already say.
