---
type: is
id: is-01m3fnf8sgc6xm43ydwwapfpnz
title: Cover language multiline literals and heredocs in code SLOC
kind: bug
status: closed
priority: 2
version: 8
spec_path: docs/project/specs/done/plan-2026-09-26-code-analysis-presentation.md
delegate: sol-code-metrics
labels: []
dependencies:
  - type: blocks
    target: is-01m3fnvw9aarv10r96g3x7tn9m
  - type: blocks
    target: is-01m3g53gjm5c6ks1t77k1az0qh
  - type: blocks
    target: is-01m3g53hk9nr8ff52d3xj77ttj
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
hold: null
hold_until: null
created_at: 2026-09-26T20:10:57.711Z
updated_at: 2026-09-27T08:18:08.932Z
started_at: 2026-09-27T01:19:37.396Z
closed_at: 2026-09-27T03:12:08.570Z
close_reason: "Implemented and versioned the streaming lexer repairs in PR133. All30 independently adjudicated syntax fixtures match; focused chunk-boundary tests and the core suite pass. Evidence: codebase-analysis-conditional-2026-09-26.json."
resolution: null
duplicate_of: null
---
Research fdu-4il8 verifies code-sloc-v1 treats comment markers inside valid multiline literals as comments: C++ raw strings, Java text blocks, C# verbatim/raw strings, Ruby/shell/SQL ordinary multiline strings, C escaped newline strings; also Ruby percent strings/heredocs, shell and PHP heredocs, SQL dollar quotes. Prioritize common syntax with compact per-language fixtures and explicit support contract; Tokei14 is correct on many but not all, so use language grammar/manual expected partitions rather than blind comparator parity. Preserve streaming/chunking/cache-version contract.
