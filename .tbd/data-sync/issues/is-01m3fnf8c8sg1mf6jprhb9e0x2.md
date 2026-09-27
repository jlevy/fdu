---
type: is
id: is-01m3fnf8c8sg1mf6jprhb9e0x2
title: Correct Rust multiline string and lifetime handling in code SLOC
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
created_at: 2026-09-26T20:10:57.275Z
updated_at: 2026-09-27T00:50:04.554Z
---
Empirical fdu 0.1.0 comparison (research fdu-4il8) confirms ordinary multiline Rust strings lose string state at newline: const S: &str = "first [newline] // text [newline] last"; is 3 code lines but reports 2 code/1 comment. A lifetime apostrophe is treated as a quote and hides a following block-comment opener: fn x<apostrophe a>() {} /* [newline] comment [newline] */ reports 3 code instead of 1 code/2 comments. Add hand-classified fixtures; preserve raw strings, nesting, mixed-line semantics and chunk boundaries. Version analyzer/cache identity if semantics change.
