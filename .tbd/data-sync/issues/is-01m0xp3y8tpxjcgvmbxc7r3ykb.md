---
type: is
id: is-01m0xp3y8tpxjcgvmbxc7r3ykb
title: "PR #47: replace surgical golden parsing with broad observable output"
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#discussion_r3858495113
    at: 2026-08-26T00:04:39.091Z
labels:
  - pr47-review
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-26T00:04:31.641Z
updated_at: 2026-08-26T00:28:12.526Z
---
The unresolved head review at tests/golden/cli-cost.tryscript.md:70 identifies a systematic golden-test anti-pattern. cli-cost runs fdu inside Node, parses its JSON and counter stream, and prints only selected booleans; several cli-content cases similarly reduce complete product output to hand-picked fields. That turns transparent-box goldens into narrow assertions, hides unanticipated changes, and duplicates behavior better pinned by ordinary tests. Fixture setup scripts and broad filesystem-state observations are not the issue. Replace report-parsing snippets with direct, broad, deterministic product output where practical; move relational cost invariants to focused Rust/integration tests or expose a compact stable diagnostic record that the golden can show in full. Audit all golden parsing sites, not only the commented line.

## Notes

Review scope confirmed at exact head 0558c7e. Audit every golden result-extraction script, especially all three Node parsers in tests/golden/cli-cost.tryscript.md and report extraction in tests/golden/cli-content.tryscript.md, while distinguishing fixture setup. Acceptance: show complete stable product output directly where practical; move relational or invariant checks to ordinary Rust/integration tests; justify any documented too-large or capture-value exception; do not replace local parsers with one shared opaque projection; run focused goldens and full CI; reply on the originating thread with a disposition map.
