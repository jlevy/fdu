---
type: is
id: is-01kzzj0c367rtcr2vxb8wrkz2w
title: "S3: store children as a sorted arena slice, superseding H7"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels: []
dependencies: []
parent_id: is-01m01mqq3cqs8ae87qd2d3rydm
created_at: 2026-08-14T07:15:27.461Z
updated_at: 2026-08-15T02:42:43.941Z
---
Every directory allocates its own BTreeMap - several nodes of allocation - for a child set that arrived together in one getdents64 batch and is then read in sorted order. A contiguous sorted slice in an arena gives one allocation per directory, binary-search lookup, and locality for the roll-up merge that walks it. Composes with S2: the slice holds (name_offset, name_len, EntryId) with names adjacent in the same arena. This supersedes H7 rather than scheduling alongside it. H7 proposed swapping BTreeMap for a hash map with a cheap hasher and has sat untested since the beginning; it changes the lookup constant but keeps the per-node allocation and loses the sorted iteration order that snapshot bytes and goldens depend on. The arena form keeps ordering, removes the allocation, and improves locality. Predict cold indexed wall down at least 5 percent and RSS down, with sorted iteration order and snapshot bytes unchanged. Index and content tiers.
