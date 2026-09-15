---
type: is
id: is-01m2gv6xvgew6qw7z73qt8e220
title: Roll up .gitignore information by default on every surface, with a per-request opt-out
kind: feature
status: open
priority: 1
version: 5
labels:
  - stack-followup
  - release
dependencies: []
created_at: 2026-09-14T20:54:48.425Z
updated_at: 2026-09-15T05:15:30.148Z
---
DECISION (user, 2026-09-14): .gitignore handling is built in and rolled up by default everywhere, and each request can turn it off.
- Engine: ScanConfig::read_controls defaults to true. execution::plan_report stops forcing it off, so one-shot reports observe and keep ignored/unignored roll-ups.
- CLI: reports and --watch observe by default; --no-gitignore turns it off.
- Python: fdu.open, fdu.scan and fdu.report observe by default, and read_controls=False turns it off.
- Opened roots: unchanged, always on.
- Snapshot scope: one default scope again (fdu-w3l5).
- The typed ControlStateNotObserved answer from #57 remains for opted-out requests.
Blocked by fdu-1onj, fdu-okne and fdu-szkg, so large .gitignore volume degrades instead of aborting. Gate: a speed check of the command `fdu PATH` against main on control-free and control-rich trees; report the numbers to the user before merging if it is more than 10% slower.

## Notes

2026-09-14 (PR #57 ab77745): the library surfaces already default on. ScanConfig::read_controls, ScanScope::default(), Index::new(root), fdu_core::open, fdu.open, fdu.scan and a watch over their index observe by default, and read_controls=false opts out, with ControlStateNotObserved answered on that index. Remaining for this bead: execution::plan_report still forces read_controls off for one-shot reports, crates/fdu/src/cli.rs still sets it off for --watch, and the CLI opt-out flag, split totals and ignored filters are not started.

2026-09-15 DECISIONS (user): ships in 0.1.0, not deferred to 0.2.0.
