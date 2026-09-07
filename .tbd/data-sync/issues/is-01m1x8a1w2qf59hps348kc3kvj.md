---
type: is
id: is-01m1x8a1w2qf59hps348kc3kvj
title: Enable shipped control semantics in performance builds
kind: bug
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T06:18:53.686Z
updated_at: 2026-09-07T06:25:27.399Z
---
Final build audit found perf-probe-release and perf-probe-profiling use --no-default-features without gitignore, so controls-enabled production/opened behavior is compiled out even on a control-rich tree. Preserve the explicit minimal-library test, but make performance builds and their probe tests enable the shipped control capability. Add a red dry-run build-contract test, verify the relevant probe tests with gitignore, document exact feature scope, and use matched controls-enabled candidate/structural binaries in final evidence. Historical b75 predates gitignore and remains an explicitly identified historical capability baseline. Do not weaken parity thresholds.

## Notes

Red dry-run tests failed for release, profiling, and debug performance recipes because each compiled out gitignore. Recipes and CI now enable the control capability while retaining a separate no-default-features probe test. The guard accepts explicit feature lists or all-features instead of freezing feature order. All 225 realtree tests pass. Full isolated make check running; main checkout's broad supply-chain scan sees unrelated ignored worktree hooks, which are preserved and excluded naturally by the isolated checkout. Final controls-enabled binaries will replace the preliminary minimal-library builds for measurement.
