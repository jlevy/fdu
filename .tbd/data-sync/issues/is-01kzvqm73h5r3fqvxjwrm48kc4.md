---
type: is
id: is-01kzvqm73h5r3fqvxjwrm48kc4
title: "PR #8: retain entries when reconciliation metadata lookup fails"
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzvqcp0wf2y0fwh6cgq16dxp
created_at: 2026-08-12T19:36:42.864Z
updated_at: 2026-08-13T05:51:06.761Z
closed_at: 2026-08-13T05:51:06.760Z
close_reason: Fixed serial and parallel reconciliation so an enumerated entry with a metadata error is retained rather than misclassified as missing; deterministic regression coverage passes.
---
The refactored portable reconciliation removes a child from the known set only after metadata succeeds. A metadata error therefore leaves an encountered name classified as missing and emits a Remove for a still-existing entry. Preserve the pre-optimization fail-closed behavior in serial and parallel paths and add deterministic regression coverage.
