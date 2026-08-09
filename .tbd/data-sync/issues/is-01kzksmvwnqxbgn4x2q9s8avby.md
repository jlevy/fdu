---
type: is
id: is-01kzksmvwnqxbgn4x2q9s8avby
title: Preserve non-UTF filesystem identity in CLI JSON
kind: bug
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzksn3gepmk01a21gkxxs6bv
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:38:05.844Z
updated_at: 2026-08-09T18:39:22.033Z
closed_at: 2026-08-09T17:58:07.414Z
close_reason: Added optional root_raw and name_raw JSON identity objects for invalid Unicode, with lowercase unix-bytes or windows-wtf16le hex payloads. Platform-focused tests prove distinct lossy-colliding names and invalid roots remain lossless; valid Unicode keeps the prior shape. make check and all goldens pass.
---
Add optional raw-name and raw-root identity metadata with documented Unix-byte and Windows-wide hexadecimal encodings. Platform tests must prove distinct invalid-Unicode names remain distinguishable while ordinary UTF-8 output stays unchanged.
