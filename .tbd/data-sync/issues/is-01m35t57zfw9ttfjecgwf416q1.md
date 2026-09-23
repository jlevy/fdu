---
type: is
id: is-01m35t57zfw9ttfjecgwf416q1
title: "PR #98 A98-R1: align Windows corpus oracle with final validity semantics"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m35sm4jn2ytfmy91136g50b7
created_at: 2026-09-23T00:20:27.753Z
updated_at: 2026-09-23T00:20:27.753Z
---
At b04c957c, corpus.py still converts zero FILETIME to -11644473600000000000 and _engine_record_bytes packs it as signed q, raising struct.error. Reproduced with the actual serializer. The oracle also retains FILE_READ_ATTRIBUTES/no denied fallback and a truncated 64-bit identity whereas the engine uses zero access/fallback and folded FileIdInfo. Update the independent oracle and add zero/out-of-range timestamp, denied/locked metadata, and 128-bit identity cases. Native FAT/exFAT and ReFS remain untested. Do before declaring the whole PR review-clear.
