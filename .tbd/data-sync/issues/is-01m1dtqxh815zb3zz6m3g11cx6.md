---
type: is
id: is-01m1dtqxh815zb3zz6m3g11cx6
title: Replace scanner ancestry overlay with a resolved-parent proof
kind: feature
status: closed
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - design
dependencies:
  - type: blocks
    target: is-01m1dtr3hap1kqbkfcap66paq8
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:33:11.463Z
updated_at: 2026-09-01T10:42:52.987Z
closed_at: 2026-09-01T10:42:52.986Z
close_reason: Private ScannerBatch and resolved-parent proof implemented; exact semantics and atomic rejection tested, exp-079 accepted, one-shot parity restored on the pinned corpus, make check and make cross-lint pass.
resolution: null
duplicate_of: null
---
Add a private owned ScannerBatch whose preparation proves canonical paths, scope, and each parent as an existing EntryId or earlier batch operation. Consume that numeric proof in detached and opened discovery so they do not build a path-keyed StructuralOverlay, while arbitrary public batches retain atomic preflight.

## Notes

Implemented a private owned ScannerBatch lane for cold and opened discovery. Preparation runs under the index write boundary, accepts only canonical unconditional discovery ops, rejects kind replacement, and resolves every parent to an existing EntryId or an earlier batch op before mutation. Application consumes the numeric proof through the shared commit/fact/control/state machinery; public, refresh, watch, and general baseline Observation paths retain the StructuralOverlay preflight. Exact engine/commit digests unchanged. Final N=12 paired run vs db18e5e: opened wall -9.50% (95% CI -10.89% to -8.14%), default wall -0.42% inconclusive/noninferior, cold wall +0.30% inconclusive/noninferior. Scoped cold allocations 162071 -> 109568 and opened 489514 -> 405751 on the stable 11141-entry corpus; ancestry overlay inserts and second-stage parent resolutions are zero. Direct N=12 comparison with the preserved pre-rewrite binary: default wall -6.07% (CI -7.68% to -0.59%); cold wall -1.50% (CI -5.64% to +0.17%), establishing restored one-shot parity. No-default and all-feature core suites, parallel/reference models, watch integration, doctests, fmt, and all-target/all-feature clippy pass. Evidence: /tmp/fdu-streaming-parity/results/run-resolved-parent-proof-final.json and run-streaming-parity-final.json. Next: commit implementation, record exp-079, update spec/ledger/report, run handoff gates.
