---
type: is
id: is-01m18r5zf7hy5bqc66pkhpa58n
title: Control-table budget aborts the scan instead of degrading to partial
kind: bug
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - scale
  - control-state
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:14.310Z
updated_at: 2026-09-13T21:52:19.650Z
---
ControlTable::upsert (crates/fdu-core/src/control.rs:120) returns Err(ControlSourceLimit) when the cumulative retained cost crosses MAX_CONTROL_TABLE_BYTES, and index.rs:1203 does the same on install. The error propagates and kills the whole scan - the user gets nothing after minutes of walking.

This contradicts the plan's own resource-limit contract, already written for max_files under 'Discovery and resource limits': reaching a limit yields partial coverage with a typed resource-budget issue, the session stays readable, and the caller reopens with a larger budget. The control table is a discovery resource budget that does not follow the contract its own design document states.

It also violates fdu-design-principles.md 'Truncate Freely; Never Truncate Silently': a hard abort is strictly worse than the truncation that rule already governs.

Fix direction: on crossing, stop retaining further control sources, mark coverage partial with a typed control-budget issue naming the affected directories, and keep the roll-up answer.

Acceptance: crossing the budget yields a usable roll-up plus an explicit partial marker; no scan aborts on control state alone; the boundary of incompleteness is knowable.

## Notes

2026-09-13 (PR #51 review follow-up): scope must include the 16 KiB per-line bound, not only the 4 MiB table bound. ControlTable::upsert returns Err(ControlPatternLimit) for any single rule over MAX_CONTROL_PATTERN_BYTES, and that aborts the scan exactly as ControlSourceLimit does; a separate review of #48 reproduced both as fatal from the default CLI (CLASS-1), and on PR #51 an engine test reproduced the per-line abort through prepare_report with the default scan config (ControlPatternLimit { attempted: 16385, limit: 16384 }).

Where each bound is still reachable after PR #51's fix (a69b95e): one-shot reports -- the command line and fdu.report -- no longer observe control state, so they reach neither bound. The opened root always observes, and fdu_core::open / fdu.open / fdu.scan and --watch observe by default, so all of them can still abort on control volume. That degradation work is this bead's; PR #51 defers it here explicitly.
