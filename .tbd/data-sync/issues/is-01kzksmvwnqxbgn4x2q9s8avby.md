---
type: is
id: is-01kzksmvwnqxbgn4x2q9s8avby
title: Preserve non-UTF filesystem identity in CLI JSON
kind: bug
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzksn3gepmk01a21gkxxs6bv
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:38:05.844Z
updated_at: 2026-08-09T17:38:13.645Z
---
Add optional raw-name and raw-root identity metadata with documented Unix-byte and Windows-wide hexadecimal encodings. Platform tests must prove distinct invalid-Unicode names remain distinguishable while ordinary UTF-8 output stays unchanged.
