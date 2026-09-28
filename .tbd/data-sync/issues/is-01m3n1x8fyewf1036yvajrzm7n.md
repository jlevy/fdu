---
type: is
id: is-01m3n1x8fyewf1036yvajrzm7n
title: "H161 residue: classification holds ~20 MiB on linux-v6.12 beyond a controls-off walk"
kind: task
status: open
priority: 2
version: 1
labels:
  - performance
  - linux
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T22:24:31.230Z
updated_at: 2026-09-28T22:24:31.230Z
---
exp-187: on linux-v6.12 the transient summary with controls on peaks at 27.6 MiB against 7.6 MiB with --no-controls (C launcher agrees), so the H161 peak-RSS cut there is -22.86% (35.8 -> 27.6), short of the pre-registered 50%, while balanced-1m (no .gitignore) is -97%. The ignored-subtree heads on that tree are few, so they do not explain it. Measure what holds it (control table sources and parsed matchers for 358 files and 1,593 rules; SummaryControls; allocator retention after 7M allocations before H162) and decide with the maintainer whether the 50% RSS criterion was meant to bind on Linux.
