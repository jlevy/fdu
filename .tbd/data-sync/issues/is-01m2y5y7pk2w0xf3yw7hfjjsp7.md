---
type: is
id: is-01m2y5y7pk2w0xf3yw7hfjjsp7
title: "PR #92 review R3: reject path aliases that undercount sidecar completeness"
kind: bug
status: closed
priority: 1
version: 5
delegate: unknown@spud10
labels: []
dependencies: []
parent_id: is-01m2y58e5s7yzpstkcqmheymy5
hold: null
hold_until: null
created_at: 2026-09-20T01:12:25.554Z
updated_at: 2026-09-20T01:26:29.636Z
started_at: 2026-09-20T01:17:56.551Z
closed_at: 2026-09-20T01:26:29.635Z
close_reason: "Addressed in f8a2ed94 on PR #92: R3 rejects noncanonical snapshot names and counts visited files; R1 owns a single-view walk; R2 covers analyzed mixed-view report semantics."
resolution: null
duplicate_of: null
---

## Notes

PR92 R3 addendum: snapshot.rs1016 accepts raw a/ alias; content_cache.rs180 map length shrinks denominator. Base refuses but19455032 accepts complete with2metadatafiles/1content. Fix shared canonical raw-name parser invariant and checksummed public cache-only regression.
