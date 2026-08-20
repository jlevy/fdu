---
type: is
id: is-01m0eyrna93ewcb5nz1jh3gncc
title: "Epic: absolute and relative performance evidence report"
kind: epic
status: closed
priority: 2
version: 6
labels: []
dependencies: []
child_order_hints:
  - is-01m0eys1v0dpb383spj6xs2ncg
  - is-01m0eys26vbvz8ma71v3j3e2pf
  - is-01m0eys2hdgvtagkww5xjdc432
  - is-01m0eys2vt0hzpsj2rgfhtxnq4
created_at: 2026-08-20T06:47:02.728Z
updated_at: 2026-08-20T07:29:06.866Z
closed_at: 2026-08-20T07:29:06.865Z
close_reason: "Shipped in PR #36 (https://github.com/jlevy/fdu/pull/36), published at https://claude.ai/code/artifact/148bc8d5-438e-4af0-acd8-8f5c2a822b93. make check and make perf-test pass."
---
Assemble the 64 soft-schema experiment artifacts into one reviewable report with charts covering both absolute timings (real milliseconds, and per-entry normalized so trees of different sizes can be compared) and relative paired effects with their intervals, plus per-experiment detail on what worked and what was rejected. Clean minimal visual design derived from the tbd web design system. Supersedes the abandoned attempt on codex/performance-research-white-paper (PR #27), which lost the absolute numbers and was overdesigned.
