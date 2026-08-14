---
type: is
id: is-01kzz3ne1z9ydzv0ztjapqdfyy
title: Linux and Windows backends for the performance profiler
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T03:04:48.959Z
updated_at: 2026-08-14T03:04:48.959Z
---
benchmarks/realtree/profile.py hard-errors on any non-Darwin host: it drives /usr/bin/sample and tells the operator to run 'perf record -g' by hand. The loop's first rule is to profile before changing anything, so on two of three supported platforms that step has no tooling. The first Linux profile in this project was taken with valgrind --tool=callgrind because perf was not installable in the container; callgrind suits user-CPU-bound targets well (it is deterministic and needs no kernel support) and is a reasonable Linux backend, with perf preferred where available. Two lessons from that run belong in whatever lands: the probe's own oracle digest was 31.9 percent of the profile and has to be attributed separately or it dominates the reading, and callgrind counts instructions rather than time so it says nothing about syscall or I/O cost.
