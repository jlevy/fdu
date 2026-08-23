---
type: is
id: is-01m00ft85qkve3wbq52c7wjjs6
title: "S1b: batch-shaped observations to remove the producer's per-entry PathBuf"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels: []
dependencies: []
parent_id: is-01m01mqq3cqs8ae87qd2d3rydm
created_at: 2026-08-14T15:56:24.105Z
updated_at: 2026-08-23T01:14:11.236Z
---
Follow-on from exp-051. The parent memo removed the consumer's descent and its component vector - normalize instructions fell 89 percent and the cold-scan-index component fell 16.6 percent - but the producer still allocates and clones a PathBuf per entry, visible as 34,256 Op::clone calls in the profile. The bead fdu-ypk2 originally proposed Op::UpsertUnder { parent: EntryId, .. }, which cannot work: EntryIds are allocated by the consumer, so a producer has no id to send. The workable form is batch-shaped: an observation carrying one directory path plus its children as (name, kind, attrs), which is exactly the grouping scan.rs already has when it builds a batch. The consumer resolves the directory once and inserts every child beneath it, so the per-entry PathBuf join and clone disappear rather than being memoized around. This is the remaining half of the gap between exp-051's -7.35 percent and the snapshot loader's -51.9 percent on the same class of defect. Predict cold-scan-index wall down a further 5 percent and Op::clone calls down to one per directory. Index tier. Re-screen S2, S3 and S4 after this lands.

## Notes

Raised by the floor measurement 2026-08-23.

Two results argue this is worth more than the ledger's framing suggests.

1. arena_spike lands at 1.06x the parallel syscall floor. Against the floor the
   representation change is not the next win in a series, it is the last one on the
   aggregate tier: after it there is nothing meaningful left in this regime.

2. The per-entry name and path handling this bead targets is where a real tree's cost
   lands. On subjects matched for entry count and directory width, swapping generated
   filenames for /usr's moves fdu from 1.35x the floor to 1.42x while moving the peer
   walker barely at all, and /usr itself sits at 1.59x. That effect is large enough to
   reverse the peer ranking against ripgrep's ignore, and it is invisible on gen_tree.py.

Consequence for measurement: this bead cannot be evaluated on a generated corpus. A
uniform tree hides roughly 15 percentage points of exactly the cost it removes.

Evidence: docs/project/reports/report-2026-08-23-metadata-walk-floor.md
