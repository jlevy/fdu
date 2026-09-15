---
type: is
id: is-01m18r6049rg359vn5nr1tazky
title: Control-table bound is not liftable by any flag and its error names no remedy
kind: bug
status: closed
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - control-state
  - cli
  - stack-followup
  - release
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:14.984Z
updated_at: 2026-09-15T20:00:42.022Z
closed_at: 2026-09-15T20:00:42.021Z
close_reason: "7d9c565, 2255593: ScanConfig::control_budget, OpenOptions::control_budget, Python ScanOptions/OpenedOptions.control_budget, and --gitignore-budget SIZE|all; None/all lifts the budget and the per-line guard together (Q3), a numeric budget keeps the guard. The budget is mixed into ignore_rules_fingerprint (Q2); the snapshot parser keeps a fixed 256 MiB ceiling; the refusal note names the knob in each surface's spelling. PR #63."
resolution: null
duplicate_of: null
---
MAX_CONTROL_TABLE_BYTES is a hard const with no CLI or config lever (verified: no match for control-table/max-control in crates/fdu/src). The error text is 'control table requires N bytes; limit is M bytes' - it states the bound and offers no way past it.

fdu-design-principles.md: 'Every bound is liftable by a flag named where the bound is stated. A truncation the caller cannot remove is a limitation wearing a default's clothes.'

A field agent independently hit exactly this and reported 'there is no flag to raise it'. --scan-depth is not a workaround: it limits scanning as well as retention, so roll-ups undercount and the answer changes. --min-size does not help either: selection applies to reporting, after the table is already built.

Fix direction: separate the two jobs the constant currently serves - keep a strict parser guard for snapshot loading (untrusted u32 lengths in snapshot.rs:688,722 must stay bounded), and add a separate, larger, flag-liftable runtime retention budget named in the error.

Acceptance: the runtime budget is settable from the CLI and named in the diagnostic; the snapshot parser guard remains strict and independent.

## Notes

2026-09-14 (triage at c0511e9): unchanged. Const at `control.rs:52@c0511e9`, no lever on `OpenOptions` (`opened.rs:95-120`) or `ScanConfig`, error text at `engine_contract.rs:1580-1582`. Reachable only where fdu-1onj is; do it after fdu-1onj, as a `DiscoveryBudget`/`ScanConfig` field named in the degraded issue.

2026-09-14 DECISION (user, supersedes the default-off decision recorded earlier the same day): .gitignore information is built into the tool and the library, and is rolled up by default on every surface: CLI reports, --watch, library open, fdu.open/fdu.scan/fdu.report, and opened roots. Each request can turn it off (--no-gitignore on the CLI, read_controls=False in the library and Python). The typed 'not observed' answer from #57 stays, for requests that opt out. The CLI shows split totals, for example '1.2 GB (340 MB ignored)', plus --exclude-ignored and --only-ignored filters. Prerequisites before the default flips: fdu-1onj (the control budget degrades to partial instead of aborting), fdu-okne (a liftable bound named in the error), fdu-szkg (charges deduplicated by fingerprint), and a speed check against main with controls on. Prerequisite for the default flip.

2026-09-15 DECISIONS (user), PR A design (plan: scratchpad/reviews/plan-gitignore-default-on.md, section 7):
Q1: a crossed control budget is a coverage note with exit 0. Sizes stay exact; only ignore classification is partial. Text output names the directories whose rules were not loaded and the flag that raises the budget. JSON carries an ignore_rules coverage field.
Q2: the budget is part of snapshot scope, mixed into ignore_rules_fingerprint. Raising it cold-scans once.
Q3: the 16 KiB per-line guard can be lifted by the same budget flag (the recommendation was fixed; the user chose liftable, per 'every bound is liftable'). Over the guard it degrades and names the guard and the flag.
Q11: bump snapshot FORMAT_VERSION in PR A, so snapshots carry refused rules and the budget.
