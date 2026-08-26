---
type: is
id: is-01m0y1sdf88k6zme7tb9hbkjrd
title: Add progressive parent-first discovery, budget, and priority
kind: feature
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex@spud10.local
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1seqtkvcawjhawny979ry
  - type: blocks
    target: is-01m0yhq8268z0qrza1fnwrddfm
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
hold: null
hold_until: null
created_at: 2026-08-26T03:28:29.671Z
updated_at: 2026-08-26T12:40:13.587Z
started_at: 2026-08-26T12:05:27.833Z
closed_at: 2026-08-26T12:40:13.586Z
close_reason: Progressive discovery, exact budget/completeness state, priority scheduling, shared admission, and deterministic coverage are implemented and validated.
resolution: null
duplicate_of: null
---
Make scan feed bounded parent-first commits to the OpenedIndex owner, record per-directory completeness, expose honest resource-budget partial state, refuse expansion after a stop, and add best-effort scheduling-only prioritize paths. Preserve one-shot scan behavior.

## Notes

Implemented parent-first progressive discovery with exact per-directory completion commits, one honest file-budget refusal boundary, bounded typed issues, scheduling-only priority hints, snapshot refusal for incomplete coverage, and shared one-shot/opened admission logic. Kept the first scheduler deliberately serial and internal: no ignored public tuning knobs, no facade, and no change to one-shot behavior. Deterministic barriers cover root-before-child ordering and priority; end-to-end tests cover settled equivalence, exact budget semantics, halted expansion, invalid roots, failures, and persistence refusal. Review dispositions: retained worker failures as typed terminal state, bounded all issue payloads, removed the active-boolean commit footgun, and extracted shared scan preparation. Validation: focused opened tests, make check, and make cross-lint all pass.
