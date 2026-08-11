---
type: is
id: is-01kzscyrcbzcq68efg7zg6qm7w
title: "Attribution: every profile must say what blocked — disk, CPU, lock-wait, channel-wait, or idle"
kind: task
status: open
priority: 1
version: 3
labels: []
dependencies: []
created_at: 2026-08-11T21:51:45.034Z
updated_at: 2026-08-11T22:08:38.861Z
---
From PR#6 design discussion. The performance loop currently reports blocked_ns as one undifferentiated I/O+sched number, so claims like 'the home folder scan is 78% blocked' are inferred rather than attributed. Instrument the probe: DirectoryQueue claim-wait ns + lock-held ns + contended-acquisition count, mpsc send-wait ns, per-worker busy/blocked/idle split. Add an attribution block to the experiment schema and render it in the ledger. Cheap: timing per claim cycle (~1.8k cycles per walk on the reference tree), never per file. This is the precondition for any scheduler change: measure what coordination costs before redesigning it.

## Notes

Core landed in 0f359c9: WalkAttribution on ScanReport (wall/work/starved/lock_wait/send + claims/lock_ops/lock_contended), both walk paths, probe top-level JSON, per-sample capture in measure.py, two identity tests. First reading on the 60k tree: work 99.57%, starvation 0.36%, lock wait 0.0014% (18/4962 contended), handoff 0.03%; claims 1,840 vs 1,835 predicted. Overhead vs pre-change binary: not detectable (warm -0.03% [-0.59,+1.37]). REMAINING: attribution block in the experiment schema + ledger rendering; revalidate sweep walk reports zeros (uninstrumented).
