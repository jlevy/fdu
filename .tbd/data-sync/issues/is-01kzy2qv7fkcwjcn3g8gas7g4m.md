---
type: is
id: is-01kzy2qv7fkcwjcn3g8gas7g4m
title: Linux cold-regime worker sweep and adaptive-calibration retune
kind: task
status: open
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - perf
  - linux
  - campaign-2
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-13T17:29:24.975Z
updated_at: 2026-08-28T15:56:25.041Z
---
Linux cold thread policy (H76/H84). Was gated on fdu-tyjx, which PR #45 closed: the aggregate-summary job now exists, so this is unblocked. Linux only -- the adaptive unlock calibrated against APFS regimes never fires on Linux; guest-cold, sixteen workers beat four by 32% at the floor itself and diskus's 3x-cores default is the whole remaining scalar-class cold gap (~22%). Bare metal (fdu-lf3v) confirms before the constant ships as evidence. Not runnable on the macOS agenda.

## Notes

2026-08-15 guest-cold sweep, 450k generated tree, 4-vCPU virtio rig (orders strategies; not device evidence): fdu summary at automatic 4 workers ~1330ms BEATS diskus -j4 ~1900ms and loses only to diskus -j12 (3x cores) ~1037ms. The gap is thread policy alone: the 30us APFS-calibrated unlock never fires against Linux's ~1.5us warm floor (H84 mechanism observed). Index tier: threads don't move it cold (4/8/12 within noise) and hurt it warm (1734/1876/1904ms) - consumer-bound, see fdu-xde5. Gated by fdu-tyjx (no aggregate probe job).

2026-08-28 (Linux session): make perf-floor (fdu-33ri, PR #49) is now the instrument this bead wants, and it is unblocked -- fdu-tyjx landed, so the aggregate tier has a probe job and a component_ns.

parfloor takes a thread count as its third argument, so the scoreboard can produce the floor's OWN scaling curve on Linux at 1/2/4/8/16 workers. That curve is the denominator the thread-policy question actually needs: "sixteen workers beat four by 32%" is a statement about fdu, and what it is missing is whether the floor scales the same way at that count on the same subject. If the floor flattens where fdu does, the constant is right and the gap is elsewhere.

Adding a --workers sweep to floor.py is a small change to the harness, not a new instrument.

Context from platform-tuning.md worth restating here: EVERY scan constant in that table has "None" under Linux evidence -- DEFAULT_SCAN_THREADS_CAP (6), ADAPTIVE_SCAN_THREADS_CAP (16), the multiplier, the calibration entry count, the slow-work threshold, DEFAULT_RECONCILE_THREADS_CAP, RECONCILE_WAVE_DIRECTORIES, DEFAULT_BATCH_SIZE. All were measured on an M1 Pro. This bead is the first one that would change that for any of them.

Caveat for whoever runs it: the bead says bare metal confirms before the constant ships as evidence, and that still holds. A 4 vCPU shared container cannot settle a worker-count knee -- it has too few cores to reach the interesting counts, and its scheduling is visibly noisy (arena_spike measured bimodal on a 76k subject there). Screening only.
