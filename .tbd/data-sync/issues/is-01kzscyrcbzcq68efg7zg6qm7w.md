---
type: is
id: is-01kzscyrcbzcq68efg7zg6qm7w
title: "Attribution: every profile must say what blocked — disk, CPU, lock-wait, channel-wait, or idle"
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-08-11T21:51:45.034Z
updated_at: 2026-08-11T22:08:28.726Z
---
From PR#6 design discussion. The performance loop currently reports blocked_ns as one undifferentiated I/O+sched number, so claims like 'the home folder scan is 78% blocked' are inferred rather than attributed. Instrument the probe: DirectoryQueue claim-wait ns + lock-held ns + contended-acquisition count, mpsc send-wait ns, per-worker busy/blocked/idle split. Add an attribution block to the experiment schema and render it in the ledger. Cheap: timing per claim cycle (~1.8k cycles per walk on the reference tree), never per file. This is the precondition for any scheduler change: measure what coordination costs before redesigning it.
