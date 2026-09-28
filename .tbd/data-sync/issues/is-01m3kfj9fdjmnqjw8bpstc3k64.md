---
type: is
id: is-01m3kfj9fdjmnqjw8bpstc3k64
title: "Code overview: show only '(N gitignored)' parentheticals, never the non-gitignored complement"
kind: bug
status: closed
priority: 1
version: 4
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
  - type: blocks
    target: is-01m3kfjb11y1k4xncszacwnaaa
parent_id: is-01m3kfj7vjnh8patz58z2p2bh1
created_at: 2026-09-28T07:44:42.987Z
updated_at: 2026-09-28T08:07:54.977Z
closed_at: 2026-09-28T08:07:54.964Z
close_reason: "Fixed in PR #144 (claude/docs-polish 41b3ae75, rustfmt 3ae0e024): code overview rows and TOTAL show only '(N gitignored)' plus ', N unknown' when non-empty; machine output unchanged; usage guide and output design state the rule; report_format tests 47/47; README examples regenerated. CI and the stack-top gate (fdu-dbvn) still to confirm."
resolution: null
duplicate_of: null
---
User report 2026-09-28: code overview rows and TOTAL render '(3,988 non-gitignored, 0 gitignored)'. The design system (docs/project/architecture/fdu-output-design.md Row Styling) puts only the gitignored amount in gray parentheses; the non-gitignored count is already the row's value, so listing it again is confusing and duplicative. Required output: '3,988 3.1% 526 358 25/25 JavaScript (0 gitignored)'; keep ', N unknown' when the unknown population is non-empty (e.g. '(20 gitignored, 10 unknown)'). Sites: crates/fdu-core/src/report_format.rs ~1682 (rows) and ~1733 (TOTAL); tests ~4701, 4732, 4792. Introduced by d59f0a39 (on main via #136, unreleased). Fix consistently across views; update the output-design doc to state the rule for the code overview, regenerate README examples (PR #143's quick start shows it), and check SKILL.md/usage docs.
