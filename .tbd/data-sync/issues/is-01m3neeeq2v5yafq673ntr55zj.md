---
type: is
id: is-01m3neeeq2v5yafq673ntr55zj
title: make check's test-performance fails where the default python3 is 3.11
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-29T02:03:37.570Z
updated_at: 2026-09-29T02:03:37.570Z
---
The Makefile runs the benchmark tests with 'uv run --no-project python' and no version, so uv takes the host default; explorations/benchmarks needs 3.12+ (test_windows_boundary_times_are_serializable raises NotImplementedError: cannot instantiate 'WindowsPath' on 3.11). Pin --python 3.12 in the recipe as release-test does. Found in the 0.2.1 stability pass.
