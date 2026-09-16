---
type: is
id: is-01m0k4rq0vhsvsk51jjb2qtk0e
title: Every bounded view states what it dropped, in text and in machine formats
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels: []
dependencies: []
parent_id: is-01m0k4qrz1rb300efa1s5z86w6
created_at: 2026-08-21T21:48:53.659Z
updated_at: 2026-09-16T16:50:15.972Z
closed_at: 2026-09-16T16:50:15.971Z
close_reason: "a6b670c (PR #39): every bounded section states 'N of M; --limit all for every one' in its header (bound_note) and carries a machine bound {shown, total} or null (bound_json) in crates/fdu-core/src/report_format.rs; Python SectionBound mirrors it."
resolution: null
duplicate_of: null
---
Every bounded view states its bound, per the truncation principle now at the top of the
design doc: "20 largest of 192,871" rather than a bare marker or nothing at all.

Covers `largest`, `recent`, and any `files` run the caller bounded with `--limit`. The
tree view already marks dropped children; this brings the flat views up to the same
contract, and names the flag that lifts it.

Machine formats need the same honesty -- a consumer reading 20 rows must be able to tell
it got 20 of 192,871 -- which means a section-level field and therefore a schema bump.
Decide the field shape alongside fdu-1lj3, which reported the original silent truncation.

## Notes

Bound statement goes in the section header, not a footer: a footer is lost to head. Name in heading colour, qualifier in telemetry colour.
