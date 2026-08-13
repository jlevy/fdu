---
type: is
id: is-01kzxsh1gqs697dy3eat13fw22
title: Refresh dut Linux source and benchmark research
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - linux
  - research
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T14:48:24.853Z
updated_at: 2026-08-13T14:58:47.952Z
closed_at: 2026-08-13T14:58:47.951Z
close_reason: null
---
Refresh the exact current dut source revision; inventory transferable mechanisms and GPL boundaries; audit its warm, page-cache-drop-only, SSD, and HDD benchmark protocols; identify semantic and correctness probes required before treating it as a comparator; and add explicit Linux experiments to the research queue, performance spec, and existing dut adapter/Linux evidence beads.

## Notes

Refreshed clean ignored attic/dut checkout to upstream 68d4ba2; reviewed README, Makefile, manpage, full main.c, and recent history. Updated the foundational survey, performance-frontier research, performance-loop H66 registry, governing performance spec/capability matrix/design-review ledger, and white paper. Added exact GPL boundary, comparator limits, Linux warm/pagecache-drop-only/controlled-cold regimes, ext4/XFS and worker sweeps, raw-reader and adapter correctness fixtures, and propagated findings to fdu-k5t5, fdu-atqk, and fdu-nffc. flowmark --auto . and make check pass.
