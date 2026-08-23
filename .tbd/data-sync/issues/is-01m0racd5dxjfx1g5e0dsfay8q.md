---
type: is
id: is-01m0racd5dxjfx1g5e0dsfay8q
title: Roll-up leaf counts so empty is decidable from the aggregate
kind: feature
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T22:03:13.964Z
updated_at: 2026-08-23T22:03:13.964Z
---
contribution() gives Symlink and Other a default rollup, so a subtree containing only symlinks is arithmetically indistinguishable from an empty one -- a listing cannot tell them apart. Maintain a non-directory leaf count (or per-kind counts) in rollup state so a complete subtree's emptiness is an exact fact from the aggregate. A partial subtree can never claim emptiness. Partition property extends to the new fields; snapshot version increments. Joins the maintained-state union priced by fdu-n4gn.
