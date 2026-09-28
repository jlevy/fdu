---
type: is
id: is-01m3k8bw0k83v0jq6zfdbvv5vj
title: "Experiment: resident fdu --watch soak with libproc open-writer list"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:52.556Z
updated_at: 2026-09-28T05:38:52.556Z
---
Rank 2. One-hour read-only soak on agent state B: resident RSS/CPU, fseventsd CPU, events/hour, classify every difference from two end walks as caught / recoverable by writer list / unexplained. Go if every stable miss is explained by an open writer or another user's process and steady CPU < 1% of a core. Also scope a lighter resident mode (cursor + dirty-dir log + per-directory totals) for home scale (~6.7M entries).
