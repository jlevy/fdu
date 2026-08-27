---
type: is
id: is-01m0y1seqtkvcawjhawny979ry
title: Add the no-gap discovery-to-observation handoff
kind: feature
status: closed
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sf2nph021wtx28p8ahxh
  - type: blocks
    target: is-01m0yhq8268z0qrza1fnwrddfm
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:30.969Z
updated_at: 2026-08-26T20:50:45.242Z
closed_at: 2026-08-26T20:50:45.225Z
close_reason: No-gap opened-root observation is implemented at 4ac5313 with the Windows-portable test follow-up at e5e27eb. Deterministic handoff, overflow/gap recovery, budget, concurrency, terminal-state, feature-disabled, and shutdown coverage pass; all 19 GitHub checks passed in run 33011941829.
resolution: null
duplicate_of: null
---
Start the supported observer before baseline capture, buffer bounded hints, reconcile overflow and registration gaps, perform required final verification, and enter watching only after freshness is proven. Reuse the scripted backend and deterministic interleaving fixtures; keep watch fully removable.

## Notes

Implemented native/scripted capture before baseline, bounded hint backlog and sticky overflow, final authoritative reconciliation, conditional watching transition, exact provider-gap freshness commits, live shared-budget observation, terminal resource-stop precedence, and joined close. Reused PR #47's scripted hint vocabulary and parser only; production coalescing, verification, admission, exact commits, reconciliation, and lifecycle remain the real path. Deterministic tests cover pre/during/post-baseline mutation, initial and live overflow, registration gaps, concurrent refresh arbitration, budget-stop races, restricted scopes, malformed scripts, cancellation, worker join, feature-disabled builds, and late observer failure after resource stop. Focused tests, 440 no-default core tests plus integration/reference-model tests, all-feature suites, and both local cross-lint targets pass. The final make check rerun passed through current-toolchain feature combinations and then hit host ENOSPC during the MSRV replay; 14.8 GiB of reconstructible FDU/Flowmark targets were staged in Trash without emptying. Clean-runner PR CI is the remaining bead gate.
