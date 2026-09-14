---
type: is
id: is-01m2esgpfc4tzvj2yfandq5cvw
title: Decide whether open and fdu.open observe control state by default
kind: task
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2eafpfpe8k5c9z9dhrqvy2y
created_at: 2026-09-14T01:46:42.539Z
updated_at: 2026-09-14T15:10:12.749Z
closed_at: 2026-09-14T15:10:12.747Z
close_reason: "c06fe47: ScanConfig::read_controls defaults off, so open/open_with_pending_save/fdu.open/fdu.scan/Index.watch observe no control state unless asked (ScanConfig::read_controls, ScanOptions.read_controls); Index::is_ignored/controls return Err(ControlStateNotObserved) on such an index; report/open/watch share one snapshot scope; planner tests now pin the warm start; docs and plan Phase 4 updated"
resolution: null
duplicate_of: null
---
Open decision left by PR #51 review COMMIT-3 (fixed in a69b95e; tracked on fdu-etfj), recorded by the fixer.

**Current behaviour after #51.** The shared one-shot planner (`crates/fdu-core/src/execution.rs` `plan_report` / `prepare_report`) always sets `ReportPlan::read_controls = false`, so `fdu <dir>` and `fdu.report()` keep controls-off snapshots. `fdu_core::open` and `fdu.open` keep `ScanConfig::read_controls` defaulting on, because the returned `Index` exposes `controls()` and `is_ignored()`. `--watch` and the opened root also observe controls. Snapshot acceptance is exact wherever an index is returned or reconciled (c0729ce), so a scanning policy treats a scope mismatch as no usable snapshot and cold-scans.

**Consequence.** An `open` or `fdu.open` right after a report on the same tree cold-scans instead of warm-starting. `51154f9` changed two planner tests to pin exactly that ("a default open no longer warm-starts from a report's snapshot"). The split is documented on `open`, `prepare_report`, `ScanConfig::read_controls`, `fdu.open`, and `fdu.report`.

**Decision.** Should `open` / `fdu.open` observe control state by default?
- **Keep on (status quo).** The index answers `controls()` and `is_ignored()` exactly, and `open` still aborts on control volume until fdu-1onj. The cost is a cold scan whenever report and open alternate.
- **Default off, opt in.** Report and open share snapshots, and `open` stops reaching the control bounds by default. But a caller who reads `is_ignored()` has to ask for it, and a controls-off index needs a clear answer for `controls()`.

The review said defaulting on is "defensible" provided the split is documented, and that it should be decided deliberately. It has been documented, not decided.

**Interacts with.** Keying snapshots by scan scope (fdu-w3l5) removes the cold-scan cost of either choice. If that lands first, this reduces to an API-semantics question. Taking "default off" also shrinks fdu-1onj's reach.

**Acceptance.** The decision is recorded in the opened-root plan's Phase 4 section and on `open` / `fdu.open`. If the default changes, the parity corpus gets the cases, the Python and CLI goldens are updated, and the planner tests are adjusted.

Review: https://github.com/jlevy/fdu/pull/51#pullrequestreview-5192254822. Disposition: https://github.com/jlevy/fdu/pull/51#issuecomment-5656491635

## Notes

2026-09-14 (fix wave, PR #51 2237a70): the command line no longer depends on this decision. `fdu --watch` now sets read_controls: false itself (crates/fdu/src/cli.rs:536-552@2237a70) because no CLI view reads control state, and one-shot reports were already off through the planner, so no `fdu` invocation observes control state or reaches a control bound whatever open's default is. Library callers still do: fdu_core::open / open_with_pending_save, fdu.open and fdu.scan (crates/fdu-py/src/lib.rs builds ScanConfig::default()), and Python Index.watch() over such an index keep observing by default and still abort on the 4 MiB table bound and the 16 KiB per-line bound (fdu-1onj). The defaults were not changed. The decision is now purely about the library API: exact controls()/is_ignored() by default, versus a shared report/open snapshot scope and an open that reaches no control bound.

2026-09-14 DECISION (user): default OFF, opt in. fdu_core::open / open_with_pending_save, fdu.open, fdu.scan and Python Index.watch() stop reading .gitignore control state unless the caller asks. On an index opened without it, is_ignored() and controls() must return a typed 'not observed', never Some(false) (today is_ignored returns Some(false) for every entry when controls were not read, index.rs:3048). Consequences: report/open/watch share one snapshot scope, and no default library call reaches the control bounds. Opened roots are unaffected; they always observe controls, so fdu-1onj degradation is still needed for them. Implement as a follow-up after the stack merges.
