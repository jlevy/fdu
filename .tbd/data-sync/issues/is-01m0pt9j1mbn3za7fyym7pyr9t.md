---
type: is
id: is-01m0pt9j1mbn3za7fyym7pyr9t
title: Bounded per-directory extension and filename rows with a remainder
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:02:49.012Z
updated_at: 2026-08-23T08:02:49.012Z
---
RollUp.by_extension is an unbounded map per directory. Metabrowser bounds its equivalents (ext_top, filename_top, remaining_top) because a browser shows a handful of rows while a wide tree has many. Add a bound with a stated remainder aggregate, same contract as the TreeNode remainder: truncate freely, never silently.
