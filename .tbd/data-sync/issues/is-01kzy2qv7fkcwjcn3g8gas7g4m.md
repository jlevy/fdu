---
type: is
id: is-01kzy2qv7fkcwjcn3g8gas7g4m
title: Linux cold-regime worker sweep and adaptive-calibration retune
kind: task
status: open
priority: 2
version: 1
labels:
  - perf
  - linux
dependencies: []
created_at: 2026-08-13T17:29:24.975Z
updated_at: 2026-08-13T17:29:24.975Z
---
Controlled-cold Linux scouting (450k entries, ext4-on-virtio, sync + echo 3 per sample): diskus (3x-cores threads) beat the fdu summary plan by 22.8% [-27.9%, -14.8%] while warm was a statistical tie - fdu under-parallelizes cold on Linux. The adaptive policy's constants (initial 4-6, 30 us/entry unlock threshold, 16-worker reserve cap) were all calibrated on APFS. The planned Linux matrix should sweep worker depth per regime and per filesystem and re-derive the service-time threshold from Linux latencies rather than inherit the APFS value. Existing plan requirement; this bead records the measured motivation.
