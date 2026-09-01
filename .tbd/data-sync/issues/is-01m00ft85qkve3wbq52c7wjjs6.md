---
type: is
id: is-01m00ft85qkve3wbq52c7wjjs6
title: "S1b: batch-shaped observations to remove the producer's per-entry PathBuf"
kind: task
status: closed
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
delegate: codex@spud10.local
labels:
  - campaign-2
dependencies: []
parent_id: is-01m01mqq3cqs8ae87qd2d3rydm
hold: null
hold_until: null
created_at: 2026-08-14T15:56:24.105Z
updated_at: 2026-09-01T18:30:39.138Z
started_at: 2026-09-01T15:20:04.362Z
closed_at: 2026-09-01T18:30:39.125Z
close_reason: "Completed by the detached directory-group bootstrap: workers publish one parent path with component-only children before descendants become claimable; controls carry one verified fixed operation per directory. Differential worker-count tests, controls fixtures, allocation evidence, and the first exact mutation prove the route."
resolution: null
duplicate_of: null
---
Follow-on from exp-051. The parent memo removed the consumer's descent and its component vector - normalize instructions fell 89 percent and the cold-scan-index component fell 16.6 percent - but the producer still allocates and clones a PathBuf per entry, visible as 34,256 Op::clone calls in the profile. The bead fdu-ypk2 originally proposed Op::UpsertUnder { parent: EntryId, .. }, which cannot work: EntryIds are allocated by the consumer, so a producer has no id to send. The workable form is batch-shaped: an observation carrying one directory path plus its children as (name, kind, attrs), which is exactly the grouping scan.rs already has when it builds a batch. The consumer resolves the directory once and inserts every child beneath it, so the per-entry PathBuf join and clone disappear rather than being memoized around. This is the remaining half of the gap between exp-051's -7.35 percent and the snapshot loader's -51.9 percent on the same class of defect. Predict cold-scan-index wall down a further 5 percent and Op::clone calls down to one per directory. Index tier. Re-screen S2, S3 and S4 after this lands.

## Notes

2026-09-01 the private cold bootstrap now sends one directory path with component-only children and builds concurrently with the shared filesystem walker. It also carries the optional verified control operation, so controls-rich scans avoid per-entry paths and repeated scanner projection while preserving parent-before-child control visibility. The latest controls-rich screen improved wall 33.55% and component 47.43% versus c6380f7 with exact worker-count differential tests. Construction now reaches pre-rewrite component parity on the current real tree. The compact retained representation and final default-tree verdict remain open under H86.
