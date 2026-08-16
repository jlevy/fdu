---
type: is
id: is-01m01pwyyczsasz1hc7aw72y2z
title: Validate exp-057 and exp-058 on macOS
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
  - macos
dependencies: []
created_at: 2026-08-15T03:19:27.436Z
updated_at: 2026-08-16T01:28:26.740Z
---
exp-057 (CRC-32C slicing-by-8, H88) and exp-058 (bootstrap journal skip, H90) were accepted on Linux measurements only; under the regime rules they are inherited, not proven, on macOS. Run the exp-054/exp-055 pattern on an APFS rig: control = pre-exp-056 main, candidate = branch tip, jobs cold-scan-index + warm-revalidate + cold-snapshot-save + warm-snapshot-load, 12+ paired interleaved trials. Also worth a screen while there: H89's rejection reasoning (exp-056) is glibc-specific - libmalloc's small-allocation path may price derive_ext differently, so the refutation is Linux-scoped too. This bead is for a macOS agent; a Linux session cannot take it.

## Notes

macOS/APFS validation run 2026-08-15, /Users/levy/wrk/aisw/trading (494,031 entries,
127,915 dirs), pr-29 head (3bda5c8 lineage) against pr-31 head (022c116), 10
counterbalanced pairs per arm, bootstrap CI on per-pair differences. Host UNCONTROLLED
(load avg 12-28) -- intervals below are noise-inclusive, which makes the confirmed one
conservative.

  cache-writing path (--cache refresh; H87 clone-share + H88 CRC exercised):
    pr29 - pr31 median -174 ms (-4.3%), 95% CI [-418, -60] -- ENTIRELY BELOW ZERO.
    user-CPU -254 ms, consistent with the synchronous index clone disappearing.
    H87/H88 CONFIRMED on APFS.

  walk-only path (--cache off; H90 exercised, H87/H88 absent):
    median -11 ms, 95% CI [-159, +133] -- tie. user-CPU -53 ms (direction right).
    H90's -5.1% was a Linux consumer-side win; on APFS the walk is kernel-bound
    (73.7% syscall frames) so the saving is real but below this host's noise floor.
    NOT a regression; re-screen on a quiet host if a number is needed (fdu-ow8y).

Caveat: pr-29 lacks PR28/PR31, so this is a cross-branch binary A/B, not a ledger
artifact -- the harness gap is fdu-ao6p. The comparison arms avoid the paths PR31
changed (refresh/off read no snapshot in either binary), so the deltas isolate the
pr-29 engine changes.
