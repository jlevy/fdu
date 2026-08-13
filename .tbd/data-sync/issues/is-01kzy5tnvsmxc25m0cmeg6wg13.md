---
type: is
id: is-01kzy5tnvsmxc25m0cmeg6wg13
title: Measure an out-of-band adaptive scale-up signal
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T18:23:23.512Z
updated_at: 2026-08-13T18:23:23.512Z
---
PR #8 senior review design observation: ScaleUp currently travels in-band behind observation batches, so consumer backpressure may delay reserve-worker creation after calibration requests it. Profile an AtomicBool or one-slot control channel against the shipped in-band design on the large cache-pressure subjects; retain only a material paired gain with identical oracle output.
