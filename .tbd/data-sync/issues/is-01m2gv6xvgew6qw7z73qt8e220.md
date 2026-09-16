---
type: is
id: is-01m2gv6xvgew6qw7z73qt8e220
title: Roll up .gitignore information by default on every surface, with a per-request opt-out
kind: feature
status: closed
priority: 1
version: 8
labels:
  - stack-followup
  - release
dependencies: []
created_at: 2026-09-14T20:54:48.425Z
updated_at: 2026-09-16T00:17:55.936Z
closed_at: 2026-09-15T22:17:59.532Z
close_reason: "dcdec5a, 3ddec4c, d108786 (PR #65): one-shot reports, fdu PATH, --watch, and fdu.report observe .gitignore by default; --no-gitignore and read_controls=False opt out; the summary tier falls closed to the index when observing (Q7); an unreadable .gitignore exits 2 unless --allow-partial (Q10), pinned in crates/fdu/tests/cli_exit.rs; speed gate passed (median pair ratios 0.92-1.04), summary RSS follow-up fdu-if7o"
resolution: null
duplicate_of: null
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

2026-09-15 DECISIONS (user), PR B design (plan: scratchpad/reviews/plan-gitignore-default-on.md, section 7):
Q4: sort and --min-size use the displayed size: total by default, unignored under --exclude-ignored.
Q5: text output uses a plain detail suffix everywhere, '(N ignored)' after the size, in summary and tree rows. No bar shading.
Q7: the transient summary tier falls closed to FullIndex so it can classify. Measure aggregate-summary wall and RSS in the speed check; if the gate fails, build the streaming classifier before merging.
Q10: an unreadable .gitignore is an operational error with exit 2 unless --allow-partial; --no-gitignore is the escape.
Recommendations taken without asking:
- Q6: zero ignored under observation omits the text suffix, and JSON carries a zero object.
- Q8: annotate extension rows in B, not types/families/languages/documents; follow up afterwards.
- Q9: the flag is --no-gitignore (the user's name).
Release: all of this ships in 0.1.0.

2026-09-15, review of PR #65 (F4, P3), on Q10. The reviewer agrees exit 2 ships as decided, and records the reasoning on both sides because the question returns when the checkpoints plan gives ignore_rules.refusals a partial marker.
Why exit 2 is not a P0 risk: the report still prints with every size exact, so only complete and the status change; a readable directory holding an unreadable .gitignore is rare, because macOS TCC and ordinary permission models protect whole directories, which already made such a result partial before this PR; and --no-gitignore is a one-flag escape. The weakness is that --allow-partial is coarse: a script that adds it to survive one mode-000 .gitignore also silences unreadable directories.
The case for modelling it as ControlRefusalReason::Unreadable instead, with exit 0 and the existing coverage note: (a) no size is missing, so complete: false overstates what went wrong; (b) the design principles separate expected coverage from operational completeness, and an unapplied rule file is coverage; (c) a refused and an unreadable .gitignore leave the same tree in the same state, and would then be reported the same way.
The case for Q10 as decided, which is what ships: every other I/O error at a path is operational, and a refusal is a policy the caller can lift where an EACCES is not.
Revisit when ignore_rules.refusals grows the checkpoints plan's partial marker, and decide then whether unreadable joins it. Pinned by crates/fdu/tests/cli_exit.rs::an_unreadable_gitignore_is_a_partial_result_that_no_gitignore_avoids.
