---
type: is
id: is-01m3neeeq2v5yafq673ntr55zj
title: make check's test-performance fails where the default python3 is 3.11
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-29T02:03:37.570Z
updated_at: 2026-09-30T02:33:45.498Z
closed_at: 2026-09-30T02:33:45.498Z
close_reason: "Fixed in 927addd4: test-performance's projectless uv run now passes --python 3.12 (as release-test does); new recipe test in scripts/check-uv-version.test.mjs (run by make supply-chain) fails on any projectless uv run without --python. Verified: reproduced 3 WindowsPath errors with UV_PYTHON unset on a 3.11 host python3, gone with the pinned command; node test fails before, passes after."
resolution: null
duplicate_of: null
---
The Makefile runs the benchmark tests with 'uv run --no-project python' and no version, so uv takes the host default; explorations/benchmarks needs 3.12+ (test_windows_boundary_times_are_serializable raises NotImplementedError: cannot instantiate 'WindowsPath' on 3.11). Pin --python 3.12 in the recipe as release-test does. Found in the 0.2.1 stability pass.
