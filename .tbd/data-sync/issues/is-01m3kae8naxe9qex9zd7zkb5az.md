---
type: is
id: is-01m3kae8naxe9qex9zd7zkb5az
title: Measure in-kernel sizing of unmarked directories via the dir-stats fsctl
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T06:15:08.201Z
updated_at: 2026-09-28T06:15:08.201Z
---
Lead from fdu-gpqz: on an unmarked 225k-entry root, fsctl 0xC1104A71 GET returned exact totals from an in-kernel walk in 1.84 s vs 7.7-9.9 s for a userspace getattrlistbulk walk (one sample, loaded host, no side effect). Replicate interleaved; compare with fdu --view summary; state its accounting rule (data-fork allocation of files whose primary link is inside; clones in full; no rsrc/xattr). Private interface: at most an optional accelerator for scalar summaries.
