---
type: is
id: is-01m0xp3y8tpxjcgvmbxc7r3ykb
title: "PR #47: replace surgical golden parsing with broad observable output"
kind: bug
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#discussion_r3858495113
    at: 2026-08-26T00:04:39.091Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#discussion_r3858670483
    at: 2026-08-26T00:28:56.979Z
labels:
  - pr47-review
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-26T00:04:31.641Z
updated_at: 2026-08-26T02:44:45.690Z
---
The unresolved head review at tests/golden/cli-cost.tryscript.md:70 identifies a systematic golden-test anti-pattern. cli-cost runs fdu inside Node, parses its JSON and counter stream, and prints only selected booleans; several cli-content cases similarly reduce complete product output to hand-picked fields. That turns transparent-box goldens into narrow assertions, hides unanticipated changes, and duplicates behavior better pinned by ordinary tests. Fixture setup scripts and broad filesystem-state observations are not the issue. Replace report-parsing snippets with direct, broad, deterministic product output where practical; move relational cost invariants to focused Rust/integration tests or expose a compact stable diagnostic record that the golden can show in full. Audit all golden parsing sites, not only the commented line.

## Notes

PR #48 review R13 repaired the dangling spec reference. This remains the Phase 1A prerequisite for replacing surgical golden parsing with broad observable-output assertions.
