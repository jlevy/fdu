---
type: is
id: is-01m1x8a1w2qf59hps348kc3kvj
title: Enable shipped control semantics in performance builds
kind: bug
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T06:18:53.686Z
updated_at: 2026-09-07T06:32:43.212Z
---
Final build audit found perf-probe-release and perf-probe-profiling use --no-default-features without gitignore, so controls-enabled production/opened behavior is compiled out even on a control-rich tree. Preserve the explicit minimal-library test, but make performance builds and their probe tests enable the shipped control capability. Add a red dry-run build-contract test, verify the relevant probe tests with gitignore, document exact feature scope, and use matched controls-enabled candidate/structural binaries in final evidence. Historical b75 predates gitignore and remains an explicitly identified historical capability baseline. Do not weaken parity thresholds.

## Notes

Full isolated make check passed with both minimal and gitignore-enabled probe unit tests, 225 realtree tests, all Rust feature combinations, MSRV, Python surfaces, goldens, docs and evidence checks. Source diff hash matches the main checkout. Correction committed and pushed; CI pending. No production algorithm, dependency, or acceptance margin changed.
