---
type: is
id: is-01kzksmmryehfvwx8beyyfppg5
title: Harden partial, color, and broken-pipe CLI contracts
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzksn3gepmk01a21gkxxs6bv
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:37:58.557Z
updated_at: 2026-08-09T18:05:55.230Z
closed_at: 2026-08-09T18:05:55.229Z
close_reason: Retained the Unix partial-result integration test; extracted deterministic color inputs and covered auto-terminal plus all suppression paths; made result finalization testable and proved contextualized broken pipes exit zero without diagnostics while complete/partial/allow-partial exit mappings stay stable. make check and 25 goldens pass.
---
Retain the Unix partial-result binary test; add deterministic color-decision and broken-pipe classification coverage that does not depend on shell timing or tryscript ANSI matching.
