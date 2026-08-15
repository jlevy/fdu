---
type: is
id: is-01kzy2qv7fkcwjcn3g8gas7g4m
title: Linux cold-regime worker sweep and adaptive-calibration retune
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - perf
  - linux
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-13T17:29:24.975Z
updated_at: 2026-08-15T02:42:44.869Z
---
Controlled-cold Linux scouting (450k entries, ext4-on-virtio, sync + echo 3 per sample): diskus (3x-cores threads) beat the fdu summary plan by 22.8% [-27.9%, -14.8%] while warm was a statistical tie - fdu under-parallelizes cold on Linux. The adaptive policy's constants (initial 4-6, 30 us/entry unlock threshold, 16-worker reserve cap) were all calibrated on APFS. The planned Linux matrix should sweep worker depth per regime and per filesystem and re-derive the service-time threshold from Linux latencies rather than inherit the APFS value. Existing plan requirement; this bead records the measured motivation.

## Notes

2026-08-15 guest-cold sweep, 450k generated tree, 4-vCPU virtio rig (orders strategies; not device evidence): fdu summary at automatic 4 workers ~1330ms BEATS diskus -j4 ~1900ms and loses only to diskus -j12 (3x cores) ~1037ms. The gap is thread policy alone: the 30us APFS-calibrated unlock never fires against Linux's ~1.5us warm floor (H84 mechanism observed). Index tier: threads don't move it cold (4/8/12 within noise) and hurt it warm (1734/1876/1904ms) - consumer-bound, see fdu-xde5. Gated by fdu-tyjx (no aggregate probe job).
