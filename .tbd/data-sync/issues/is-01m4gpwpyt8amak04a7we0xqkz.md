---
type: is
id: is-01m4gpwpyt8amak04a7we0xqkz
title: Measure a large Python report's cost from eager modified_at datetimes
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-09T16:10:43.033Z
updated_at: 2026-10-10T18:49:14.930Z
started_at: 2026-10-10T18:48:39.792Z
closed_at: 2026-10-10T18:49:14.930Z
close_reason: "Measured: eager modified_at costs about 1.45 us a row, 145 ms of a 100,000-row list report and 1.4% of the fdu.report call, which the engine's 10 s list view dominates. A lazy property on the frozen, slotted FileRow would save at most that 1.4%; not worth changing the model now. Reopen if the engine's list view gets fast enough for the Python conversion to dominate."
resolution: null
duplicate_of: null
---
Review C7 fix 3 on #191 (exp-212, exp-213): Python's _models.py derives every row's modified_at datetime eagerly from mtime_ns. Time a 100k-row list report through fdu.report before and after the age fields, and if the derivation is a measurable share, make it lazy (a cached property) without changing the public type.

## Notes

Measured 2026-10-10 (exp-217's evidence, python-modified-at.json): a 100,000-row list report on linux-balanced-1m, fdu.report(root, Query(views=(View.LIST,), format=Format.JSON, selection=Selection(limit=100000))), Python 3.12, wheels built from 148ef78e and 0d6011da, quiet, 20 pairs each.
- The modified_at derivation: json.loads + report_from_dict takes 1,298 ms with it and 1,142 ms with _modified_at stubbed, a paired median of 145 ms, about 1.45 us a row (pairs range -176 to +318 ms), about 11% of the model conversion.
- The whole call: 10.6 s at 0.5.0 against 11.7 s at 0.4.0 (-7.1% paired), so the derivation is about 1.4% of it. The call is the engine's: the native report_once takes 11.4 s and the CLI 10.0 s for the same list, against 1.2 s for the default tree over the same tree (new bead).
Note: Format.TEXT with View.LIST returns a tree projection with no flat rows, so a list benchmark must ask for a flat format.
