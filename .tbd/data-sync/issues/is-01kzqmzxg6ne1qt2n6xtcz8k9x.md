---
type: is
id: is-01kzqmzxg6ne1qt2n6xtcz8k9x
title: "P1: value grammars parse_when and parse_size"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn07pd0n9fvf00r6ate71f
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:33:42.789Z
updated_at: 2026-08-11T05:33:53.228Z
---
New feature-independent crates/fdu/src/query/parse.rs. parse_when(s, now: SystemTime) implements the spec WHEN grammar exactly: 'now' | compound ages (45s, 2h, 1h30m; units s/sec/second(s), m/min(s)/minute(s), h/hr(s)/hour(s), d/day(s), w/week(s)) | RFC3339 (offset-carrying; round-trips scan_started_at exactly) | local 'YYYY-MM-DD [HH:MM[:SS]]' | @epoch with optional fraction. Rejections carry suggestions: calendar units -> 30d/365d, fractional ages -> compounds (1.5h -> 1h30m), natural language rejected entirely; @epoch is the only legal fraction. parse_size accepts 10M / 1.5GiB (decimal and binary). No new dependency: humantime is unmaintained (RUSTSEC-2025-0014); jiff is the documented fallback only if scope outgrows this. now is a parameter so callers and tests control the reference instant. Tests: table-driven over every accepted form and every rejection with its exact message.
