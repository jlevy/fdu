---
type: is
id: is-01kzzj10r2ddn2vg9f676pcebm
title: "S4: build a cold bootstrap without arbitration, sharpening H60"
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T07:15:48.609Z
updated_at: 2026-08-14T07:15:48.609Z
---
The single mutation authority exists so snapshots, queries and change feeds cannot diverge and concurrent producers cannot race. Both are real concerns for a WARM path. A cold scan has no prior state to arbitrate against, no concurrent readers and no present-state ABA to reject - it constructs a tree from nothing - yet every entry still crosses a channel as an allocated observation, is applied by one serialized consumer, and merges its contribution to the root. Linux measured the consequence: fdu's index consumer costs about 2.3 microseconds per entry of user CPU against dut's 0.1, roughly twenty times, and that gap is the whole tree-class deficit. No syscall change reaches it; the enumeration layer is already at parity. H60 points here but frames it as a construction optimization. The sharper framing is that the cold path can drop arbitration entirely: workers build disjoint subtrees in local arenas with no coordination, splice at region boundaries, and one bottom-up pass computes every roll-up - removing the channel, the per-entry observation allocation, the consumer serialization, and merge_upward's O(N*D) together. Largest and riskiest item; attempt only after S1-S3, which are cheaper and will shrink it. Prove parity with the existing assert_same_image differential harness at every worker count. Predict cold indexed wall down at least 20 percent on Linux. Index tier.
