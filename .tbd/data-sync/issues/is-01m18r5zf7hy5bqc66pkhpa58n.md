---
type: is
id: is-01m18r5zf7hy5bqc66pkhpa58n
title: Control-table budget aborts the scan instead of degrading to partial
kind: bug
status: open
priority: 0
version: 10
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - scale
  - control-state
  - stack-followup
  - release
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:14.310Z
updated_at: 2026-09-15T05:15:27.783Z
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

2026-09-13 (PR #48 verification review 5193206420, FIX48-2/-5/-7): scope widened, and two interim gaps narrowed.

What still fails on a control bound, which the CLASS-1 disposition understated (FIX48-7). The discovery improvement reaches only roots that do not observe. (a) A watched root -- MetaBrowser's mode, and fdu_core::open / fdu.open / --watch by default -- still ends Failed: discovery degrades past the refused directory and reaches Ready, then the observation handoff's full reconcile (reconcile_paths_handle_controlled over the root) applies the same control, gets the same ControlSourceLimit or ControlPatternLimit as an error, and run_observation returns it, so the root fails later, after an extra full walk, and close() names the observation worker instead of discovery. (b) In steady state, once the cumulative table bound has been reached, the first event touching any .gitignore-bearing directory fails the observer the same way: reconcile_pending_target propagates the commit error (scan.rs reconcile_pending_target Err arm), apply_next_controlled returns it, and the observation worker ends with the root Failed. Acceptance for this bead should include both: a watched root over the bound reaches Watching with a typed partial marker, and a steady-state event over a control-bearing directory degrades instead of failing the observer.

Interim changes on PR #48 (commits 9c29e6f for FIX48-2 and FIX48-5): item (1) above is narrowed -- a refusal now still queues the subdirectories that earlier batches of the same listing had committed, so only the refused batch's entries and the rest of that listing are lost, not subtrees nothing refused. Item (2) is narrowed -- the refusal is retained as a ResourceBudget issue whose path is the root-relative directory whose control crossed the bound, rather than a pathless ProviderFailure; the coverage reason is still Inaccessible, and the issue names the directory, not the control file. Still owned here: retrying the refused batch without its control op (the review's preferred option for FIX48-2), so the directory's ordinary entries survive and the directory can complete with a typed marker, plus items (3)-(5) and (a)-(b). Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5193206420

2026-09-14 (fix wave, PR #51 2237a70): `fdu --watch` no longer reaches either bound. The command line's scan configuration now sets read_controls: false (crates/fdu/src/cli.rs:536-552@2237a70). Verified first that nothing under --watch consumes control state: the session drops ControlUpdated and Reclassified (watch_session.rs:202), the CLI Selection and every report view and renderer read no ignore classification (EntrySelection.exclude_ignored is opened-root only), nothing under crates/fdu/src calls is_ignored or controls(), and no golden depends on it. Pinned by crates/fdu/tests/watch_controls.rs, which drives the binary and was red before the change with "control pattern requires 16385 bytes; limit is 16384 bytes": a watch over one 16,385-byte rule serves a complete initial report, and a watch whose .gitignore is edited past the bound keeps applying later changes and saves a snapshot the next watch starts warm from. All 125 goldens pass unchanged. The opened-root plan's Phase 4 statements that --watch observes control state and can abort on it are corrected.

Still owned here, unchanged by that commit: fdu_core::open / open_with_pending_save, fdu.open, fdu.scan, and Python Index.watch() over such an index observe by default (the default is fdu-agb6) and still abort on either bound in the blocking scan_into_index path (pinned by scan.rs detached_control_bootstrap_matches_control_limit_failures); the opened root's residuals (1)-(5) and the watched-root failures (a)-(b) above. Side effect for fdu-okne: no command-line surface reaches a control bound any more, so its 'liftable from the command line' half has no CLI caller until something on the CLI observes control state again.

2026-09-14 (fdu-agb6, c06fe47 on claude/contract-decisions): the default no longer reaches either bound from a library call. ScanConfig::read_controls defaults off, so fdu_core::open / open_with_pending_save, fdu.open, fdu.scan and a watch over their indexes read no control file unless the caller opts in. Still reaching both bounds: every opened root (read_controls is always on there, opened.rs OpenOptions::into_parts) and any open or scan that opts in. This bead stays open for those.

2026-09-14 DECISION (user, supersedes the default-off decision recorded earlier the same day): .gitignore information is built into the tool and the library, and is rolled up by default on every surface: CLI reports, --watch, library open, fdu.open/fdu.scan/fdu.report, and opened roots. Each request can turn it off (--no-gitignore on the CLI, read_controls=False in the library and Python). The typed 'not observed' answer from #57 stays, for requests that opt out. The CLI shows split totals, for example '1.2 GB (340 MB ignored)', plus --exclude-ignored and --only-ignored filters. Prerequisites before the default flips: fdu-1onj (the control budget degrades to partial instead of aborting), fdu-okne (a liftable bound named in the error), fdu-szkg (charges deduplicated by fingerprint), and a speed check against main with controls on. This bead is now a prerequisite for the default flip. Without it, `fdu ~` would abort again on large .gitignore volume.
