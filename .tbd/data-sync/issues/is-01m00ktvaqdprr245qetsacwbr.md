---
type: is
id: is-01m00ktvaqdprr245qetsacwbr
title: Adopt samply as the cross-platform profiler, closing the Linux and Windows gap
kind: task
status: open
priority: 1
version: 2
labels:
  - campaign-2
dependencies: []
created_at: 2026-08-14T17:06:38.039Z
updated_at: 2026-08-23T09:09:04.917Z
---
Closes fdu-fz3j. Profiling today is callgrind on Linux and a bespoke macOS-only script (benchmarks/realtree/profile.py), so two of three supported platforms cannot take the first step of the performance loop, and the loop's own rule is to profile before changing anything. samply (v0.13.1) runs one command on all three - perf on Linux, DTrace on macOS, ETW on Windows - and opens the result in the Firefox Profiler, which reads as a normal sampling profiler with a caller tree. That would replace the bespoke macOS script with a maintained tool and give Windows profiling for the first time. It does not replace callgrind: callgrind gives deterministic instruction counts and exact caller trees where samply gives statistical samples, so the two answer different questions - samply for 'where does wall time go on this platform', callgrind for 'exactly how many instructions did this change move'. Keep both and document which to reach for.
