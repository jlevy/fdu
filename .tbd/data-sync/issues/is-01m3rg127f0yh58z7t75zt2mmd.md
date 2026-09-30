---
type: is
id: is-01m3rg127f0yh58z7t75zt2mmd
title: "Atomic writes: old research scripts (explorations/fsevents-replay, change-sources, yaml-conformance)"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m3rfz36eh8m7xp52aq5c0kmz
created_at: 2026-09-30T06:28:59.247Z
updated_at: 2026-09-30T07:13:13.420Z
closed_at: 2026-09-30T07:13:13.419Z
close_reason: "Research scripts in explorations/change-sources, fsevents-replay, yaml-conformance write results through scripts/atomic_write.py (sys.path bootstrap to the repo root) or scripts/atomic-write.mjs (node_parse.mjs); fsevents run.save and real_tree.atomic_json now use the shared helper (atomic_json keeps its budget, dir fsync, 0600). Append-only JSONL streams (replay-cost runs.jsonl, writer-coverage live-ops, watch-soak livewatch.jsonl): new atomic_write.complete_lines drops a torn last record, used by every Python reader. Workload writes (the changes under study) are listed as INPUT_WRITERS in scripts/check-atomic-writes.mjs; drive.py and ops.py JSONL appends listed as exceptions. Evidence: make atomic-writes passes with PENDING removed; every bootstrap resolves to the repo root; no new pyflakes/bugbear findings; fsevents real_tree (11) and run (4) self-tests pass. Commit acb2882b."
resolution: null
duplicate_of: null
---
About 95 plain writes in one-off research scripts; convert mechanically to the shared helper. Lowest priority of the epic.
