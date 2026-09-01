---
type: is
id: is-01kzzj0bqfxfgxakh7a0xhanqd
title: "S2: hold entry names in one arena instead of allocating each twice"
kind: task
status: in_progress
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
delegate: codex@spud10.local
labels:
  - campaign-2
dependencies: []
parent_id: is-01m01mqq3cqs8ae87qd2d3rydm
hold: null
hold_until: null
created_at: 2026-08-14T07:15:27.086Z
updated_at: 2026-09-01T16:55:03.290Z
started_at: 2026-09-01T15:20:04.391Z
---
Each entry owns name: OsString and its parent's children map owns the same bytes again: two heap allocations and two copies per entry for one name. H19-H22 mentions removing the duplication but has never been measured, and the stronger form is not in the registry: one growable byte arena for the whole index with entries holding (offset u32, len u16), which is what fsearch does. Takes per-entry name allocation from two to zero and makes sibling names contiguous. Aimed at the largest line in both profiles - the allocator is about 35 percent of cold-scan engine work and was 27.5 percent of snapshot-load work before fdu-91ts. Composes with S1 rather than competing: S1 removes path allocations, this removes name allocations. Predict million-entry RSS down at least 20 percent and cold indexed wall down at least 3. Index and content tiers.

## Notes

2026-09-01 the incoming child name is moved into Entry and cloned once for the parent key, and consumed directory lookup paths are retired. This removed about one allocation per entry: controls-disabled scoped allocations are 923,671 versus 1,107,018 at c6380f7. A single retained name arena remains part of the full H86 representation; implement only with the child-slice and promotion boundary so partial conversion costs are not reintroduced.
