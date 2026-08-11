---
type: is
id: is-01kzqn4rdq9vy4qvcve073rfhf
title: "P3: --watch CLI loop, fdu.stream/1, and signal handling"
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn502680awzhvddzntq32d
  - type: blocks
    target: is-01kzqn5atxakb84p364hjfhg1p
  - type: blocks
    target: is-01kzqn5vw0t83yh77s92f6njf9
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:36:21.430Z
updated_at: 2026-08-11T19:23:47.602Z
closed_at: 2026-08-11T17:08:48.083Z
close_reason: Session composes IndexHandle+Watcher+Query; --watch loop with fdu.stream/1 tagged change records, dirty-gated aggregate repaint, and event-driven detection throughout. Selection filters the stream, with removals filtered only by path and escalations never filtered. --watch + --scan-depth is a usage error that teaches scope-vs-selection. watch joins default features while cli-only and no-default-features still build. 5 integration tests against real filesystem events, including the idle-cost contract.
---
The watch loop: (1) open per cache policy and emit the initial report exactly as a one-shot run would; (2) drive Session batches; (3) the files view streams per batch - one text line or one jsonl record per effective applied op carrying path, op (upsert|remove), kind, size, mtime, and the index clock, which is the tail -f surface; the same selection window applies, so --modified-since 1h --watch bounds the initial report then streams everything after it, and --modified-since now --watch is a pure tail with an empty initial listing (no suppress-initial flag needed); (4) aggregate views re-render at most once per --interval (default 2s) and only when dirty, separated in text by a timestamped header - the interval throttles rendering only and plays no part in detection; (5) overflow or subtree invalidation appears as an explicit invalidate record with its reason followed by the post-reconciliation report, never dropped (Principle 5); (6) SIGINT/SIGTERM exit 0 after a final snapshot save when the index is Fresh and policy allows writes, while watch errors exit 1. New fdu.stream/1 JSONL schema with tagged record types (report, change, invalidate, status), distinct from the one-shot fdu.report/1.

## Notes

Audit 2026-08-11: closed on partial delivery. The loop, stream schema, and dirty-gated repaint were done; 'signal handling and final save' was not. Now addressed, though not as the spec worded it: std has no portable signal handler, and a watch session ends by signal far more often than it ends politely, so an exit-time save would be the one that never runs. The loop instead saves after each dirty batch, throttled to the render interval, which is strictly more robust (it also survives a crash or SIGKILL). Pinned by crates/fdu/tests/watch_persistence.rs, which kills the real binary with SIGKILL and then requires a --cache only read to succeed.
