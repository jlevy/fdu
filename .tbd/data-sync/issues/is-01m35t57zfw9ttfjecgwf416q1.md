---
type: is
id: is-01m35t57zfw9ttfjecgwf416q1
title: "PR #98 A98-R1: align Windows corpus oracle with final validity semantics"
kind: bug
status: in_progress
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@local
labels: []
dependencies: []
parent_id: is-01m35sm4jn2ytfmy91136g50b7
created_at: 2026-09-23T00:20:27.753Z
updated_at: 2026-09-23T01:21:54.927Z
---
At b04c957c, corpus.py still converts zero FILETIME to -11644473600000000000 and _engine_record_bytes packs it as signed q, raising struct.error. Reproduced with the actual serializer. The oracle also retains FILE_READ_ATTRIBUTES/no denied fallback and a truncated 64-bit identity whereas the engine uses zero access/fallback and folded FileIdInfo. Update the independent oracle and add zero/out-of-range timestamp, denied/locked metadata, and 128-bit identity cases. Native FAT/exFAT and ReFS remain untested. Do before declaring the whole PR review-clear.

## Notes

Acceptance implemented on codex/alpha-windows-oracle: independent oracle uses zero-access opens; denied/locked listing fallback with size, write time and unavailable identity; zero/saturated FILETIME conversion; full FileIdInfo fold with legacy 64-bit fallback. Local focused corpus 36 pass (2 native Windows skips), full corpus 78 pass (2 skips), realtree 309 pass (1 skip), perf probe 16 pass. Native FAT/exFAT, ReFS and real denied/locked fixture still need Windows validation on PR. Existing spec: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md.
