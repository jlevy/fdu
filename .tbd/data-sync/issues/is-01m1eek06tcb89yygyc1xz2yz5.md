---
type: is
id: is-01m1eek06tcb89yygyc1xz2yz5
title: Compact one-shot batches and fixed partitions
kind: task
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01m1dtr903vj783j9ajaxfnczf
hold: null
hold_until: null
created_at: 2026-09-01T12:20:01.882Z
updated_at: 2026-09-01T12:40:20.363Z
started_at: 2026-09-01T12:20:05.907Z
closed_at: 2026-09-01T12:40:20.360Z
close_reason: Rejected and reverted under the preregistered H99 gate. exp-085 records default-tree -2.56% CI [-3.33%, -0.13%], below the 3% structural threshold; cold-scan-index -1.63% CI [-2.74%, +0.34%]. Scanner-only transport was neutral, so the composite did not justify its 490-line representation and materialization paths.
resolution: canceled
duplicate_of: null
---
Pre-registered H99 structural composite. The completed-index heap trace names a second representation cost: scanner-owned batches retain Vec<ObservationOp> even though every scanner operation is unconditional; at the H98 high-water mark those buffers accounted for 25.6 MB, while the pre-rewrite walker transported compact Vec<Op>. Combine (1) compact ScannerBatch Vec<Op> with conversion to ObservationOp only at the public scan streaming boundary and (2) the rejected exp-084 optional fixed-partition representation/control-free reducer lane. Primary accept gate: metabrowser-current default-tree wall improves at least 3% versus 3c0e1a2 with the paired 95% interval below zero. Secondary gates: cold-scan-index moves in the same direction; public scan and opened discovery preserve exact operations/digests/control semantics; final allocation, reallocation, and requested-byte ratios against b75bf85 are each <=1.05; opened-discovery has no >3% wall/component upper bound and no unbounded allocation/RSS regression. Reject and revert the complete composite if any gate fails.
