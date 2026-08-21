---
type: is
id: is-01m0k4rq0vhsvsk51jjb2qtk0e
title: Every bounded view states what it dropped, in text and in machine formats
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels: []
dependencies: []
parent_id: is-01m0k4qrz1rb300efa1s5z86w6
created_at: 2026-08-21T21:48:53.659Z
updated_at: 2026-08-21T21:50:34.098Z
---
Every bounded view states its bound, per the truncation principle now at the top of the
design doc: "20 largest of 192,871" rather than a bare marker or nothing at all.

Covers `largest`, `recent`, and any `files` run the caller bounded with `--limit`. The
tree view already marks dropped children; this brings the flat views up to the same
contract, and names the flag that lifts it.

Machine formats need the same honesty -- a consumer reading 20 rows must be able to tell
it got 20 of 192,871 -- which means a section-level field and therefore a schema bump.
Decide the field shape alongside fdu-1lj3, which reported the original silent truncation.
