---
type: is
id: is-01m44y1836pjj2m15kg0nj8wgg
title: Native reader test stale_bytes_in_the_buffer_never_reach_a_listing failed once on CI (MSRV 1.85), not reproduced
kind: bug
status: open
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-10-05T02:24:38.500Z
updated_at: 2026-10-09T05:22:59.142Z
---
`scan::linux_dents::tests::stale_bytes_in_the_buffer_never_reach_a_listing` failed once in CI, in the MSRV (1.85) job of https://github.com/jlevy/fdu/actions/runs/37254623682, on PR #169's head 5c6b4016:

    StatPolicy { skip_dir_symlink_stat: true, one_filesystem: true }: a reused buffer changed the listing

The same engine code passed in every other CI job and run, including MSRV on 705d3825. The failure came in the last policy's third read, the latest read in the test, about 34 s into a 912-test suite on a 4-vCPU runner.

Not reproduced locally (Linux 6.18, ext4 and tmpfs):
- 600 runs of the test binary built with Rust 1.85, four processes at once;
- 150 directory layouts, varying the names to vary ext4's hash order, each read under all four policies with a fresh, a garbage-filled and a reused buffer;
- every make check on this code.

Ruled out by reading the code: the H185 searchability latch is per listing, and under this policy a directory is always statted and a symlink is always answered with default attrs, so the latch cannot make two reads differ. On this host's ext4, a 1-byte file reports 8 blocks immediately after the write and after sync, so delayed allocation does not change `allocated` here.

Open: whether an attribute changes over time on the runner's filesystem (allocated blocks, or ctime or mtime granularity), or the reader has a real nondeterministic fault.

Next step: make the assertion name the entries and fields that differ (fresh-only and reused-only, with their attrs), so the next occurrence says which field moved. If it is an attribute that changes between reads, compare the kind and name sets exactly and the attrs only where the filesystem holds them stable, or settle the files (sync) before the first read. If it is the reader, fix it.

## Notes

Recurred 2026-10-09 on Engine on Linux arm64 at #189 head 2a643cfd (scan::linux_dents::tests::stale_bytes_in_the_buffer_never_reach_a_listing: 'a garbage-filled buffer changed the listing', linux_dents.rs:1103); the PR changes no crate; the job passed at f5b5ac4b and on main a6b441c4. Rerun with --failed.
