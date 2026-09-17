---
type: is
id: is-01m2pj0gs0cyxp178kz5kktpbb
title: Shared adversarial YAML conformance corpus, tested with real parsers in CI
kind: task
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - output
  - testing
dependencies:
  - type: blocks
    target: is-01m2pj0hxv9y019pxwhnvpgkx8
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
created_at: 2026-09-17T02:09:27.839Z
updated_at: 2026-09-17T02:57:34.361Z
---
Strings that must round-trip as strings: number forms in every base (0x10, 1_000, 0b101, 0o17, .inf,
.NaN), YAML 1.1 booleans (y, n, on, off), sexagesimal (12:30:00), dates, indicators (: # - ? * & ! % @ `),
quotes, leading/trailing space, newline, tab, DEL and C1 controls including NEL, U+2028/U+2029, BOM,
U+FFFE, non-UTF-8 paths. Run fdu output through PyYAML, ruamel and npm yaml (both versions) and compare
with JSON; reuse the corpus in jlevy/frontmatter-format.

## Notes

121-string corpus and parser harnesses (uv PyYAML/ruamel, node yaml 1.1/1.2) saved at attic/yaml-conformance-eval-2026-09-17 (local, gitignored); a 3000-string fuzz found the LS/PS-beside-space case.
