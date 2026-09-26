---
type: is
id: is-01m3fnf8sgc6xm43ydwwapfpnz
title: Cover language multiline literals and heredocs in code SLOC
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-26T20:10:57.711Z
updated_at: 2026-09-26T20:10:57.711Z
---
Research fdu-4il8 verifies code-sloc-v1 treats comment markers inside valid multiline literals as comments: C++ raw strings, Java text blocks, C# verbatim/raw strings, Ruby/shell/SQL ordinary multiline strings, C escaped newline strings; also Ruby percent strings/heredocs, shell and PHP heredocs, SQL dollar quotes. Prioritize common syntax with compact per-language fixtures and explicit support contract; Tokei14 is correct on many but not all, so use language grammar/manual expected partitions rather than blind comparator parity. Preserve streaming/chunking/cache-version contract.
