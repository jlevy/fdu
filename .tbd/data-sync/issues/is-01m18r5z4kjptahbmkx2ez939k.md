---
type: is
id: is-01m18r5z4kjptahbmkx2ez939k
title: Control state is built for every scan, including roll-ups that never use it
kind: bug
status: closed
priority: 0
version: 7
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - scale
  - control-state
  - stack-followup
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:13.970Z
updated_at: 2026-09-14T14:37:37.369Z
closed_at: 2026-09-14T14:37:37.368Z
close_reason: Triage 2026-09-14 recommended closing on merge to main. The control-observation gate (a69b95e, planner read_controls=false, plus CLI --watch off in 2237a70) is on main via stack merge 7cc5554. The library open default is decided on fdu-agb6.
resolution: null
duplicate_of: null
---
Root cause of the control-table aborts.

crates/fdu/Cargo.toml:47 sets default = ["watch", "gitignore"], and crates/fdu-core/src/scan.rs calls read_control_op() at every walk site with no runtime gate - only the compile-time gitignore feature. So a plain 'fdu ~ -d 1 --sort size' opens, reads, parses, and retains every .gitignore in the tree, then dies on a budget for state the roll-up never consumes.

The control table exists to serve the opened-root inventory (MetaBrowser partitioning). A size roll-up needs none of it.

Fix direction: gate control observation on a runtime capability - build it when a consumer asks for ignore classification, not unconditionally. This alone makes 'fdu ~' work irrespective of the cap, and removes thousands of small-file reads from every scan.

Acceptance: a default CLI roll-up performs no control-file I/O and retains no control state; opened-root/inventory consumers still get exact control state; no-default-features build unaffected.

## Notes

Implemented on PR #51 (897d8fe): ScanConfig.read_controls, gated read_control_op funnel, CLI one-shot off / watch on / opened always-on, scope identity shared with the compiled-out capability, directional snapshot acceptance so watch-written caches still serve one-shot reads. Acceptance met for the default CLI: file opens = 0 on a 304-gitignore tree, no control state retained. Bead closes when the PR merges.

2026-09-13 (PR #51 review 5192254822, COMMIT-3 and PLAN-1/PLAN-3; PR #50 review PLAN-3): the "acceptance met" note above covered one surface of three. The policy lived in cli.rs, so fdu.report() -- which reaches the same planner with ScanConfig::default() -- still read and retained every .gitignore and could still abort on the control bounds, and the CLI and Python wrote snapshots of different scope to one cache path.

Fixed on PR #51 in a69b95e: ReportPlan::read_controls (always false; no report view reads ignore classification) and prepare_report runs the scan, snapshot scope check, report scope, and save under it whatever the caller passed; the CLI no longer sets the field. fdu_core::open / fdu.open keep observing by default (the index exposes controls() and is_ignored()), and the resulting cache split is documented on open, prepare_report, ScanConfig::read_controls, and fdu.open / fdu.report. 51154f9 updates two planner tests that had pinned the old shared scope. Plan text corrected in b36d5aa.

Acceptance by surface now: CLI one-shot and Python fdu.report -- no control-file I/O, no retained control state (engine test a_one_shot_report_observes_no_control_state_whatever_the_caller_configured, red before the fix with ControlPatternLimit; the binding calls fdu_core::prepare_report and is covered by the Python and parity CI jobs). Opened root, open, --watch -- exact control state, still able to abort on control volume until fdu-1onj. Note: the earlier "counters show zero file opens" evidence could not show this either way, because the file_opens counter counts only content-analysis opens. CI 19/19 green at 51154f9. Bead still closes when the PR merges.

2026-09-13 (stack-followup audit): "closes when the PR merges" above means when PR #51's control-observation gate reaches main through the stack, not when #51 merges into its base branch. Do not close it before then. Nothing else is owed here, since acceptance by surface is recorded above.

What this fix leaves open is tracked elsewhere:
- the open/fdu.open default: fdu-agb6;
- report, open, and --watch snapshots evicting each other: fdu-w3l5;
- opened-root, open, and --watch aborts on control volume: fdu-1onj.

2026-09-14 (triage at c0511e9): verified at the combined head. `plan_report` forces `read_controls: false` (`execution.rs:184@c0511e9`) and `prepare_report_internal` applies it before the scan (`execution.rs:243-249`); both front ends reach that planner (`cli.rs:586`, `fdu-py/src/lib.rs:1276`). Close on merge to main.
