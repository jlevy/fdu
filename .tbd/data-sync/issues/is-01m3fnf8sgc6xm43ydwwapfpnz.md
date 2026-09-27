---
type: is
id: is-01m3fnf8sgc6xm43ydwwapfpnz
title: Cover language multiline literals and heredocs in code SLOC
kind: bug
status: open
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m3fnvw9aarv10r96g3x7tn9m
  - type: blocks
    target: is-01m3g53gjm5c6ks1t77k1az0qh
  - type: blocks
    target: is-01m3g53hk9nr8ff52d3xj77ttj
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
created_at: 2026-09-26T20:10:57.711Z
updated_at: 2026-09-27T00:50:04.566Z
---
Research fdu-4il8 verifies code-sloc-v1 treats comment markers inside valid multiline literals as comments: C++ raw strings, Java text blocks, C# verbatim/raw strings, Ruby/shell/SQL ordinary multiline strings, C escaped newline strings; also Ruby percent strings/heredocs, shell and PHP heredocs, SQL dollar quotes. Prioritize common syntax with compact per-language fixtures and explicit support contract; Tokei14 is correct on many but not all, so use language grammar/manual expected partitions rather than blind comparator parity. Preserve streaming/chunking/cache-version contract.
