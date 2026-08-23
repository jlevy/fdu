---
type: is
id: is-01m01mrdztdny8d1k9ar9b012p
title: "H93: profile-guided optimization on release builds"
kind: task
status: open
priority: 3
version: 4
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
  - campaign-2
  - macos-agenda
dependencies: []
created_at: 2026-08-15T02:42:01.850Z
updated_at: 2026-08-23T10:07:11.705Z
---
Fat LTO and codegen-units=1 are taken; PGO on the branchy consumer typically returns 5-15% with zero source change. One afternoon experiment: instrument, run scan-index+revalidate workloads, rebuild with profile, measure under the loop. Pre-registered signal: cold-scan-index and warm-revalidate wall down >=3%; if accepted, wire into release pipeline only.

## Notes

2026-08-23 overnight: skipped, not attempted. llvm-tools is not installed on this host (PGO needs rustup's llvm-profdata matching rustc), and the host is uncontrolled tonight -- the engine guard run's intervals were +/-13-23 points -- while the effect PGO can show on macOS is bounded by the user-space share of a warm wall, about 25%, so a 5-15% user-space gain is 1-4% of wall: unresolvable under this noise. Run on a quiet host with 16-20 trials after 'rustup component add llvm-tools'.
