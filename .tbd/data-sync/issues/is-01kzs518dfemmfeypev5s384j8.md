---
type: is
id: is-01kzs518dfemmfeypev5s384j8
title: Compose provenance through roll-ups
kind: task
status: open
priority: 0
version: 6
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzs51acramxq68xg5k81n2d7
  - type: blocks
    target: is-01kzs52fyqhmvw2dz2jkm4hqd4
  - type: blocks
    target: is-01kzs52haatwf30skqvs3vd3p1
  - type: blocks
    target: is-01kzs5yx7v7xan384vcwyznv7q
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:33:18.325Z
updated_at: 2026-08-11T19:49:29.974Z
---
RollUp gains worst-source, oldest-observation and worst-status, composed in merge/unmerge exactly like every other roll-up field, so a directory is only as trustworthy as its least trustworthy descendant. Reuses merge_upward rather than adding machinery. Also add Index::provenance(path) constructing the view type from the stored byte plus the index-level timestamps. Property test: composition is monotone (adding a worse child never improves a parent) and matches a brute-force recomputation over the subtree.

## Notes

DESIGN NOTE from implementing fdu-ywa4: provenance composes by max (weakest source, worst status) and min (oldest observation), which is NOT invertible - RollUp::unmerge cannot undo it. The codebase already has this exact problem and its solution: newest_mtime_ns is also a max, and removals call recompute_newest_upward, stopping early once a directory's value is unchanged so the common case stays O(depth). Provenance needs the same treatment. Also still open from fdu-ywa4: revalidation must stamp UNCHANGED entries as Revalidated, which today it does not - apply_upsert only sets source when allocating a new entry, and an unchanged entry produces no mutation at all, so verified entries currently keep reporting Cached.
