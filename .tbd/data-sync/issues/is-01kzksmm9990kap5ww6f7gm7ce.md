---
type: is
id: is-01kzksmm9990kap5ww6f7gm7ce
title: Correct by-type metric and output-mode semantics
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzksn3gepmk01a21gkxxs6bv
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:37:58.056Z
updated_at: 2026-08-09T17:50:46.949Z
closed_at: 2026-08-09T17:50:46.948Z
close_reason: Added failing goldens then made the by-type summary use apparent bytes consistently and made --by-type conflict with --json at argument parsing. Exact human and usage-error transcripts pass.
---
Add the failing by-type cases, make the entire human by-type report use apparent bytes, and reject --by-type --json as an explicit usage conflict instead of silently ignoring the flag.
