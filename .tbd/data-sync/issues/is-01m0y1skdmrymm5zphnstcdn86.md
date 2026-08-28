---
type: is
id: is-01m0y1skdmrymm5zphnstcdn86
title: Measure final performance, dependency, and size acceptance
kind: task
status: open
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
child_order_hints:
  - is-01m10nshe4qenm5s8ce206xa3k
  - is-01m10nshrq8ska2thptbjmp8vs
  - is-01m10nsj426pyks0x8h9azvfka
created_at: 2026-08-26T03:28:35.764Z
updated_at: 2026-08-28T15:24:18.267Z
---
Measure cold usefulness and completion, settled query and continuation work, change latency, CPU, memory, dependency trees, CLI binary size, wheel size, and GIL boundary cost on the same corpus. Publish exact revisions and regimes and record the explicit rollback/default-provider decision without changing defaults in this bead.

## Notes

## Command-line size growth is explained, 2026-08-28 — not a regression

Recorded earlier as an open question on the grounds that +297,632 raw bytes (+11.8%) in
a binary that never opens a root looked like reachable code that should not be reachable.
Measuring the denominator answers it.

| | `main` | HEAD | Growth |
| --- | --- | --- | --- |
| `fdu-core` source lines | 28,280 | 43,709 | **+54%** |
| `fdu` command-line source lines | 2,516 | 2,516 | 0% |
| Command-line binary, raw bytes | 2,523,264 | 2,820,896 | **+11.8%** |
| Command-line binary, gzip bytes | 1,115,576 | 1,254,332 | +12.4% |

The engine grew by more than half; the binary that links it grew by about a ninth. Growth
roughly a fifth of the source growth is link-time elimination working as intended, not
failing: the command line pays for the shared parts it genuinely reaches — the exact
commit pipeline every producer now routes through, classification, the query machinery —
and not for opened-root serving, which it never constructs.

The engine's own isolation test says the same thing from the other direction:
`detached_indexes_never_allocate_or_populate_serving_state` asserts a detached index
carries no serving state at all, and the command line's 125 goldens are unchanged.

The plan's rule is that an *unexplained* regression blocks the design. This one is
explained and proportionate, so it does not. Retaining the numbers here as the recorded
before-and-after this bead owes, rather than as an objection.

Still owed by this bead, and untouched by the above: wheel and binding bytes, cold
startup, one-shot scan time, peak memory, change latency, and GIL boundary cost, under
the paired protocol.
