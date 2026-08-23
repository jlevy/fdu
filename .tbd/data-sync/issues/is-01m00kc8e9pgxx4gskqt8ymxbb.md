---
type: is
id: is-01m00kc8e9pgxx4gskqt8ymxbb
title: Every read_dir costs two getdents64 calls; half are pure end-of-stream detection
kind: task
status: open
priority: 4
version: 5
labels: []
dependencies:
  - type: blocks
    target: is-01kzg49rw1p40pjc18feb9ghpv
created_at: 2026-08-14T16:58:39.945Z
updated_at: 2026-08-23T01:52:09.154Z
---
Found by the three-tier cross-check the instrumentation playbook prescribes, and invisible to any single tier. Application counters report 2,559 directory opens for the 17,128-entry content tree. strace on the same job reports 2,565 openat, 17,131 statx - both matching the app counters within noise - and 5,118 getdents64, which is exactly 2.00 per read_dir. The second call in each pair returns zero to signal end of directory: std's ReadDir cannot know the directory is exhausted until the kernel says so, so half of all directory-read syscalls carry no data. At the 29 microseconds per call strace measured, that is roughly 74 milliseconds on this tree, and it scales with directory count rather than entry count - so it hurts most on the wide shallow trees that are common in practice. Worth investigating whether a raw getdents64 loop can skip the confirming call when the returned buffer is shorter than the one supplied, which is the usual trick, and whether that is actually safe across filesystems - a short return does not strictly guarantee exhaustion on every filesystem, and getting this wrong silently truncates a directory listing, so any implementation needs a differential test against the std walker over a tree with directories sized either side of the buffer boundary. Relates to H77 and to the macOS bulk reader, which already bypasses std for the same class of reason. This is also the worked example the playbook's cross-check section should carry: an application counter that is accurate about what the code called, and wrong about what the kernel did.

## Notes

Priced against a measured floor 2026-08-23; recommend demotion.

parfloor (benchmarks/spikes/parfloor.c) puts the ENTIRE enumeration layer -- every
openat, both getdents64, every close, and the name copies -- at 0.144 us/entry on a
4-vCPU Linux VM, which is 60.3 ms of the aggregate tier's 216.5 ms wall. The terminating
call is a fraction of that and returns no data. Bounding it generously at half the
per-directory syscall cost puts it under 1% of aggregate-tier wall on a 21-wide tree and
under 4% on a 5-wide one.

That is below the 3% accept gate before any of the statfs f_type safety analysis this is
blocked on. walkspike's elide variant already showed the mechanism works (57,260 calls to
28,630, tallies identical, warm wall inside noise). Compose it into a dir-heavy or cold
campaign if it is done at all; do not run it as its own experiment.

Evidence: docs/project/reports/report-2026-08-23-metadata-walk-floor.md
