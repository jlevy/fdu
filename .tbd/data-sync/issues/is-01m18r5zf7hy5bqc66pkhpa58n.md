---
type: is
id: is-01m18r5zf7hy5bqc66pkhpa58n
title: Control-table budget aborts the scan instead of degrading to partial
kind: bug
status: open
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - scale
  - control-state
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:14.310Z
updated_at: 2026-09-13T22:57:28.160Z
---
ControlTable::upsert (crates/fdu-core/src/control.rs:120) returns Err(ControlSourceLimit) when the cumulative retained cost crosses MAX_CONTROL_TABLE_BYTES, and index.rs:1203 does the same on install. The error propagates and kills the whole scan - the user gets nothing after minutes of walking.

This contradicts the plan's own resource-limit contract, already written for max_files under 'Discovery and resource limits': reaching a limit yields partial coverage with a typed resource-budget issue, the session stays readable, and the caller reopens with a larger budget. The control table is a discovery resource budget that does not follow the contract its own design document states.

It also violates fdu-design-principles.md 'Truncate Freely; Never Truncate Silently': a hard abort is strictly worse than the truncation that rule already governs.

Fix direction: on crossing, stop retaining further control sources, mark coverage partial with a typed control-budget issue naming the affected directories, and keep the roll-up answer.

Acceptance: crossing the budget yields a usable roll-up plus an explicit partial marker; no scan aborts on control state alone; the boundary of incompleteness is knowable.

## Notes

2026-09-13 (PR #51 review follow-up): scope must include the 16 KiB per-line bound, not only the 4 MiB table bound. ControlTable::upsert returns Err(ControlPatternLimit) for any single rule over MAX_CONTROL_PATTERN_BYTES, and that aborts the scan exactly as ControlSourceLimit does; a separate review of #48 reproduced both as fatal from the default CLI (CLASS-1), and on PR #51 an engine test reproduced the per-line abort through prepare_report with the default scan config (ControlPatternLimit { attempted: 16385, limit: 16384 }).

Where each bound is still reachable after PR #51's fix (a69b95e): one-shot reports -- the command line and fdu.report -- no longer observe control state, so they reach neither bound. The opened root always observes, and fdu_core::open / fdu.open / fdu.scan and --watch observe by default, so all of them can still abort on control volume. That degradation work is this bead's; PR #51 defers it here explicitly.

2026-09-13 (PR #48 review CLASS-1, deferred here; opened-root status after the LIFE-1 fix in c801d4e): the review reproduced both bounds as fatal from the default CLI -- a synthetic tree of 1,105 directories each holding a 510-byte .gitignore ended cold scan_into_index with ControlSourceLimit (retained cost is roughly 64 + dir + 2*bytes + 64*(lines+1) per file, so about 1,370 repository-root-style or 5,000 package-style files exhaust the 4 MiB table), and a single 16,385-byte line anywhere ended it with ControlPatternLimit (MAX_CONTROL_PATTERN_BYTES is 16 KiB). The walk also reads .gitignore files inside directories that are already ignored, which git never does (scan.rs cold walk), and one .gitignore over 4 MiB is a non-fatal scan error while the cumulative and per-line bounds are fatal. One-shot surfaces are closed by PR #51's control-observation gate plus its COMMIT-3 follow-up, so #48 must not merge to main without #51.

For opened roots, a control-bound error no longer kills discovery; it now degrades, but only partly. discover_directory classifies a refused commit (opened.rs discovery_rejection): ControlSourceLimit and ControlPatternLimit refuse that one directory's listing, which stays incomplete, and the refusal is retained as a ProviderFailure issue through an Inaccessible transition, so coverage is Partial(Inaccessible) and discovery continues with every other directory. Pinned by opened::tests::a_control_bound_refuses_one_directory_without_ending_discovery. What this bead still owns: (1) the whole batch is dropped with the control, so that directory's ordinary entries and subdirectories are lost too, rather than only the control; (2) the issue does not name the control path, and the coverage reason says Inaccessible rather than a control or budget reason; (3) the observation handoff's full reconcile applies the same controls and still fails on the same bound, so a watched root still ends Failed; (4) controls under already-ignored directories are still read and retained; (5) the unignored partition's knowledge below a dropped control is not marked incomplete. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101
