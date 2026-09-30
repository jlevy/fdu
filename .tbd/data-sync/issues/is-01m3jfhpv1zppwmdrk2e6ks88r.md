---
type: is
id: is-01m3jfhpv1zppwmdrk2e6ks88r
title: Refresh the editable native extension before Python tests
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-27T22:25:09.472Z
updated_at: 2026-09-30T02:45:24.723Z
closed_at: 2026-09-30T02:45:24.722Z
close_reason: "Duplicate of fdu-35b1, fixed once in ba4a09a9: python-check rebuilds the editable extension (--reinstall-package fdu) before pytest, reusing cargo's incremental artifacts rather than building a wheel. See fdu-35b1 for why cache keys were not used and what still needs the coordinator's make check."
resolution: null
duplicate_of: null
---
Local make check can run Python source against a stale editable _native extension after native binding changes. During fdu-n4ow, six tests failed because Python passed render(format,color,bar_size) but the cached extension still accepted two arguments. The independently built current wheel passed; refreshing the editable extension made all 70 Python tests pass. Make python-check explicitly establish a current native build or verify its source identity before pytest, without duplicating wheel builds unnecessarily. Preserve external scratch/build policy and test the stale-to-current transition. CI clean builds do not reproduce the cached local environment.
