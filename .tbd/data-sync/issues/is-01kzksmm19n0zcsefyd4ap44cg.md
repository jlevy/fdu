---
type: is
id: is-01kzksmm19n0zcsefyd4ap44cg
title: Specify CLI invocation, errors, help, and human output as goldens
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzksn3gepmk01a21gkxxs6bv
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:37:57.800Z
updated_at: 2026-08-09T18:39:21.350Z
closed_at: 2026-08-09T17:50:46.730Z
close_reason: Added exact end-to-end surface and human sessions for complete help/version, default root, stdout/stderr/exit contracts, fatal errors, stable tree output, sorting, bars, depth, and per-directory number limits. Expanded help with scan-versus-view semantics and exit statuses; all goldens and make check pass.
---
The surface and human sessions lock full help/version output, default empty-tree behavior, fatal and usage errors, stream separation, ordering, bars, depth, number, and apparent-size rendering. Expand --help to document exit statuses and scan-versus-view limits.
