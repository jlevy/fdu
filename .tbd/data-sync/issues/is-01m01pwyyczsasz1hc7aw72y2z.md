---
type: is
id: is-01m01pwyyczsasz1hc7aw72y2z
title: Validate exp-057 and exp-058 on macOS
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
  - macos
dependencies: []
created_at: 2026-08-15T03:19:27.436Z
updated_at: 2026-08-15T03:19:27.436Z
---
exp-057 (CRC-32C slicing-by-8, H88) and exp-058 (bootstrap journal skip, H90) were accepted on Linux measurements only; under the regime rules they are inherited, not proven, on macOS. Run the exp-054/exp-055 pattern on an APFS rig: control = pre-exp-056 main, candidate = branch tip, jobs cold-scan-index + warm-revalidate + cold-snapshot-save + warm-snapshot-load, 12+ paired interleaved trials. Also worth a screen while there: H89's rejection reasoning (exp-056) is glibc-specific - libmalloc's small-allocation path may price derive_ext differently, so the refutation is Linux-scoped too. This bead is for a macOS agent; a Linux session cannot take it.
