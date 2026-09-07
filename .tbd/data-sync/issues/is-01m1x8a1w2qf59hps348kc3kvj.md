---
type: is
id: is-01m1x8a1w2qf59hps348kc3kvj
title: Enable shipped control semantics in performance builds
kind: bug
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
  - type: blocks
    target: is-01m1xawdxr2v87f7km4jabmd8x
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T06:18:53.686Z
updated_at: 2026-09-07T07:12:58.350Z
closed_at: 2026-09-07T06:46:47.802Z
close_reason: 1a39be9 passed all 19 CI checks, the complete fresh-target make check, and both installed cross-lint targets. The guard and CI preserve minimal-library testing and require control-capable performance binaries.
resolution: null
duplicate_of: null
---
Final build audit found perf-probe-release and perf-probe-profiling use --no-default-features without gitignore, so controls-enabled production/opened behavior is compiled out even on a control-rich tree. Preserve the explicit minimal-library test, but make performance builds and their probe tests enable the shipped control capability. Add a red dry-run build-contract test, verify the relevant probe tests with gitignore, document exact feature scope, and use matched controls-enabled candidate/structural binaries in final evidence. Historical b75 predates gitignore and remains an explicitly identified historical capability baseline. Do not weaken parity thresholds.

## Notes

Full isolated make check passed with both minimal and gitignore-enabled probe unit tests, 225 realtree tests, all Rust feature combinations, MSRV, Python surfaces, goldens, docs and evidence checks. Source diff hash matches the main checkout. Correction committed and pushed; CI pending. No production algorithm, dependency, or acceptance margin changed.
