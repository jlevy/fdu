---
type: is
id: is-01m0y1sf2nph021wtx28p8ahxh
title: Expose the five synchronous OpenedIndex operations in Python
kind: feature
status: closed
priority: 1
version: 12
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sfedr9qf3sc7e4bf6fd7
  - type: blocks
    target: is-01m0y1sjbfs5h264xhme2vqymg
  - type: blocks
    target: is-01m10nq3f65ssfz0jj2nkxavrn
  - type: blocks
    target: is-01m10nsdjx7z9h87m4nf8hzhyh
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
child_order_hints:
  - is-01m10nq2bqxrkqxrtkh7rs668g
  - is-01m10nq2s4pvy11rrdrjxyzv2k
  - is-01m10nq345g2dt9hqmxq7kyvrg
created_at: 2026-08-26T03:28:31.316Z
updated_at: 2026-08-27T05:35:13.193Z
closed_at: 2026-08-27T05:35:13.193Z
close_reason: Implemented and verified the direct opened-root Python surface at 0583a1a/fa85812. Full make check, cross-lint, installed wheel/sdist lifecycle and typing, CLI/Python parity, MSRV, and the complete GitHub Actions matrix all pass. The standalone CLI raw stripped size is unchanged and its golden corpus remains green; no runtime dependency was added.
resolution: null
duplicate_of: null
---
Add PyOpenedIndex and immutable Python models for open, read, changes, refresh, prioritize, and close. Release the GIL around blocking or substantial native work, preserve shared close semantics, avoid a package-owned async executor, update stubs and typing, and prove overlap and shutdown.

## Notes

Execution is split into fdu-seku (total PyO3 value/handle binding), fdu-2fhv (immutable public fdu-native models, stubs, and thin wrapper), and fdu-nsn3 (GIL, lifecycle, one-shot surface parity, typing, sdist, and installed-wheel proof). MetaBrowser is the reference client but contributes no public Python vocabulary; the CLI remains an independent peer over the same core and Python never shells out to it. Close this checkpoint only after all three children and the fdu Python handoff gate pass.
