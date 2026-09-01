---
type: is
id: is-01m00ft85qkve3wbq52c7wjjs6
title: "S1b: batch-shaped observations to remove the producer's per-entry PathBuf"
kind: task
status: in_progress
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
delegate: codex@spud10.local
labels:
  - campaign-2
dependencies: []
parent_id: is-01m01mqq3cqs8ae87qd2d3rydm
hold: null
hold_until: null
created_at: 2026-08-14T15:56:24.105Z
updated_at: 2026-09-01T16:10:18.204Z
started_at: 2026-09-01T15:20:04.362Z
---
Follow-on from exp-051. The parent memo removed the consumer's descent and its component vector - normalize instructions fell 89 percent and the cold-scan-index component fell 16.6 percent - but the producer still allocates and clones a PathBuf per entry, visible as 34,256 Op::clone calls in the profile. The bead fdu-ypk2 originally proposed Op::UpsertUnder { parent: EntryId, .. }, which cannot work: EntryIds are allocated by the consumer, so a producer has no id to send. The workable form is batch-shaped: an observation carrying one directory path plus its children as (name, kind, attrs), which is exactly the grouping scan.rs already has when it builds a batch. The consumer resolves the directory once and inserts every child beneath it, so the per-entry PathBuf join and clone disappear rather than being memoized around. This is the remaining half of the gap between exp-051's -7.35 percent and the snapshot loader's -51.9 percent on the same class of defect. Predict cold-scan-index wall down a further 5 percent and Op::clone calls down to one per directory. Index tier. Re-screen S2, S3 and S4 after this lands.

## Notes

2026-09-01 prototype evidence: the controls-disabled cold path now sends one directory path with component-only child records and constructs the private index concurrently. It avoids per-file PathBuf construction and public Observation wrappers. A naive post-walk consolidation was ~50% slower because it lost pipeline overlap; the pipelined form plus direct roll-up contribution reached +0.19% median, CI [-2.96%, +1.78%], against c6380f7 in a 12-pair uncontrolled exploratory run. Full H86 acceptance and the compact retained representation remain open.
