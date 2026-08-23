---
type: is
id: is-01m01pwz9zn8qedryq786y5ygh
title: Hardware CRC32C behind runtime detection (ARMv8 / SSE 4.2)
kind: task
status: open
priority: 3
version: 3
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
  - macos
  - campaign-2
  - macos-agenda
dependencies: []
created_at: 2026-08-15T03:19:27.807Z
updated_at: 2026-08-23T09:09:09.574Z
---
Follow-up split out of fdu-e446 (closed with the accepted slicing-by-8 form): the polynomial is a hardware instruction on both target ISAs, reachable with core::arch intrinsics plus runtime feature detection and counters-style unsafe confinement. The ARMv8 half is the macOS-measurable one (Apple Silicon), so this belongs to a macOS agent first; screen only if the digest becomes a larger share again (e.g. after H92 grows snapshot payloads). Pre-registered signal unchanged from H88: snapshot save/load component down >=3% over the slicing baseline.
