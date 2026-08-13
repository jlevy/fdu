---
type: is
id: is-01kzvr6zbtkrhgsvv8sqfnghwk
title: Harden macOS bulk record name references
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzvqcp0wf2y0fwh6cgq16dxp
created_at: 2026-08-12T19:46:57.529Z
updated_at: 2026-08-13T05:51:07.205Z
closed_at: 2026-08-13T05:51:07.204Z
close_reason: Bulk parser now rejects backward name references into fixed metadata fields and has bounds-focused regression coverage.
---
The bulk parser bounds-checks name offsets against the record but currently accepts a malformed name reference that points backward into fixed metadata fields if those bytes happen to form a valid NUL-terminated component. Enforce that variable name data begins after all fixed fields and add a parser regression test so the documented fail-closed contract is complete.
