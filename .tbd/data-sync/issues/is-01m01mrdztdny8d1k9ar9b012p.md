---
type: is
id: is-01m01mrdztdny8d1k9ar9b012p
title: "H93: profile-guided optimization on release builds"
kind: task
status: open
priority: 3
version: 3
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
  - campaign-2
  - macos-agenda
dependencies: []
created_at: 2026-08-15T02:42:01.850Z
updated_at: 2026-08-23T09:09:09.319Z
---
Fat LTO and codegen-units=1 are taken; PGO on the branchy consumer typically returns 5-15% with zero source change. One afternoon experiment: instrument, run scan-index+revalidate workloads, rebuild with profile, measure under the loop. Pre-registered signal: cold-scan-index and warm-revalidate wall down >=3%; if accepted, wire into release pipeline only.
