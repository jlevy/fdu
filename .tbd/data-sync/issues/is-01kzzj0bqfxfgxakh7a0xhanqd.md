---
type: is
id: is-01kzzj0bqfxfgxakh7a0xhanqd
title: "S2: hold entry names in one arena instead of allocating each twice"
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T07:15:27.086Z
updated_at: 2026-08-14T07:15:27.086Z
---
Each entry owns name: OsString and its parent's children map owns the same bytes again: two heap allocations and two copies per entry for one name. H19-H22 mentions removing the duplication but has never been measured, and the stronger form is not in the registry: one growable byte arena for the whole index with entries holding (offset u32, len u16), which is what fsearch does. Takes per-entry name allocation from two to zero and makes sibling names contiguous. Aimed at the largest line in both profiles - the allocator is about 35 percent of cold-scan engine work and was 27.5 percent of snapshot-load work before fdu-91ts. Composes with S1 rather than competing: S1 removes path allocations, this removes name allocations. Predict million-entry RSS down at least 20 percent and cold indexed wall down at least 3. Index and content tiers.
