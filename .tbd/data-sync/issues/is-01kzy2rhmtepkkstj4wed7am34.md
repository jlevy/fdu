---
type: is
id: is-01kzy2rhmtepkkstj4wed7am34
title: Record Linux io_uring statx refutation and gate fdu-ktka on bare-metal cold evidence
kind: chore
status: open
priority: 3
version: 1
labels:
  - perf
  - linux
dependencies: []
created_at: 2026-08-13T17:29:47.930Z
updated_at: 2026-08-13T17:29:47.930Z
---
Scouting spike (kernel 6.18, io_uring enabled, QD-128 IORING_OP_STATX per directory, hand-rolled ring): +327% wall [+309%, +345%] and 4.4x CPU warm; +77.6% wall [+72.7%, +92.4%] and 2.3x CPU controlled-cold, versus plain statx on the same walk. No getdents opcode exists mainline, so enumeration cannot ride the ring either. Recommendation: treat fdu-ktka (io_uring accelerator) as closed on current evidence; reopen only if a bare-metal high-latency cold run (real NVMe/network fs, not host-cached virtio) shows the sign flipping. The macOS bulk-call gap has no profitable Linux analog today - Linux wins must come from fewer stats (tiers) and cheaper user-space, not batched syscalls.
