---
type: is
id: is-01kzqwkhq65rv9y9ngw960b4xr
title: Track benchmark decisions across reproducible environment cells
kind: feature
status: closed
priority: 1
version: 6
labels:
  - performance
  - benchmark
  - ci
dependencies: []
created_at: 2026-08-11T07:46:46.117Z
updated_at: 2026-08-11T08:50:56.440Z
closed_at: 2026-08-11T08:50:56.439Z
close_reason: Implemented the v3 cross-environment harness and manual Ubuntu workflow; archived equivalent macOS/APFS and Linux/ext4 env-001 runs; published fail-closed per-cell decisions and report; fixed Windows portability; local make check and all 13 PR checks pass at b85b744; final review posted at https://github.com/jlevy/fdu/pull/4#issuecomment-5251009709. Controlled concurrency and launcher-independent Linux RSS remain tracked by fdu-wfvx.
---
Make the real-tree benchmark harness rerunnable on cloud Linux and other hosts, identify each host/filesystem/runner as a separate evidence cell, recompute acceptance decisions per cell, compare only equivalent workloads and revisions, and publish the Linux-versus-Mac outcome in PR #4.

## Notes

Pre-commit review: R1 full job contracts/variant args; R2 recomputed portable workload identity; R3 runner class bound to evidence grade/provider; R4 temporary PR workflow trigger removed after the cloud artifact; R5 degenerate GitHub Linux wait4 RSS failed closed. R1-R5 fixed. CI review found R6: a new Linux-filesystem unit test asserted a POSIX path spelling on Windows; prior CI also exposed one freshly-created timestamp fixture race. The path assertion now compares the platform-native Path string and the reference-tree fixture pins deterministic mtimes. Targeted tests, 20 repeated fingerprint comparisons, all 87 real-tree tests, and make check pass. env-001 is path-free, deterministic, zero-invalid; fdu-wfvx owns controlled fixed-concurrency repetition plus launcher-independent Linux RSS.
