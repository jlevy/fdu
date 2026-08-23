---
type: is
id: is-01kzy2qv7fkcwjcn3g8gas7g4m
title: Linux cold-regime worker sweep and adaptive-calibration retune
kind: task
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - perf
  - linux
  - campaign-2
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-13T17:29:24.975Z
updated_at: 2026-08-23T09:09:39.183Z
---
Linux cold thread policy (H76/H84). Was gated on fdu-tyjx, which PR #45 closed: the aggregate-summary job now exists, so this is unblocked. Linux only -- the adaptive unlock calibrated against APFS regimes never fires on Linux; guest-cold, sixteen workers beat four by 32% at the floor itself and diskus's 3x-cores default is the whole remaining scalar-class cold gap (~22%). Bare metal (fdu-lf3v) confirms before the constant ships as evidence. Not runnable on the macOS agenda.

## Notes

2026-08-15 guest-cold sweep, 450k generated tree, 4-vCPU virtio rig (orders strategies; not device evidence): fdu summary at automatic 4 workers ~1330ms BEATS diskus -j4 ~1900ms and loses only to diskus -j12 (3x cores) ~1037ms. The gap is thread policy alone: the 30us APFS-calibrated unlock never fires against Linux's ~1.5us warm floor (H84 mechanism observed). Index tier: threads don't move it cold (4/8/12 within noise) and hurt it warm (1734/1876/1904ms) - consumer-bound, see fdu-xde5. Gated by fdu-tyjx (no aggregate probe job).
