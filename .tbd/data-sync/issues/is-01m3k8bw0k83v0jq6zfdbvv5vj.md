---
type: is
id: is-01m3k8bw0k83v0jq6zfdbvv5vj
title: "Experiment: resident fdu --watch soak with libproc open-writer list"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:52.556Z
updated_at: 2026-09-28T06:52:46.671Z
closed_at: 2026-09-28T06:52:46.670Z
close_reason: "Soak done 2026-09-27 (explorations/change-sources/watch-soak/): fdu --watch on agent state B (476k entries) for 61 min. Final view exact (0 stable misses, 0 false positives) but only because every macOS rename escalated to a full-root reconcile (174/h, 48% of a core, RSS median 442 MB, 6.9 GB/h of snapshot rewrites). Event-only counterfactual: 99.8% of in-place growth bytes were in held-open files; the libproc writer re-stat (45 ms) recovers all. Follow-ups: fdu-822y (rename escalation bug), fdu-88p7 (persistence rewrites), fdu-eru0 (startup revalidation), fdu-d2iz (dirty-directory recorder)."
resolution: null
duplicate_of: null
---
Rank 2. One-hour read-only soak on agent state B: resident RSS/CPU, fseventsd CPU, events/hour, classify every difference from two end walks as caught / recoverable by writer list / unexplained. Go if every stable miss is explained by an open writer or another user's process and steady CPU < 1% of a core. Also scope a lighter resident mode (cursor + dirty-dir log + per-directory totals) for home scale (~6.7M entries).
