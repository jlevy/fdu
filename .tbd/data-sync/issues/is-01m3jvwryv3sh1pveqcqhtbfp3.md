---
type: is
id: is-01m3jvwryv3sh1pveqcqhtbfp3
title: "Spike: real-root FSEvents cached refresh against full-scan oracle"
kind: task
status: closed
priority: 1
version: 6
delegate: claude-code
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:00:54.990Z
updated_at: 2026-09-28T16:20:58.682Z
started_at: 2026-09-28T02:02:02.625Z
closed_at: 2026-09-28T02:36:06.196Z
close_reason: "Research spike completed in commit 494169b8 / PR 131. Full make check, make cross-lint, focused probe tests, strict Python checks, and all applicable PR CI checks passed. Prior-art audit and sanitized real-root and 26-hour replay evidence committed. Production acceptance remains unproven: busy-root conservative scopes erase speed benefit and all 16 next-day trials time out before HistoryDone. Remaining implementation and replay-completion investigation stay open under fdu-uwhl."
resolution: null
duplicate_of: null
---
Extend the committed standalone replay probe to read existing roots without modifying them. Capture metadata baseline and pre-scan cursor, exit, replay after real activity, reconcile dirty scopes, persist candidate independently, then compare against full metadata oracles with concurrent changes explicitly classified. Test the user-selected agent-state roots; use external scratch and retain sanitized evidence. Review prior research and modern native references. No production engine or public API change.

## Notes

Implemented real_tree.py, parent-reviewed no-follow traversal/private external state/resource bounds/failure preservation. Six focused tests pass on Python 3.12; Ruff and project-strict BasedPyright pass. Parent strict check found and fixed three annotation/summary typing issues missed by the agent default config; a verified reversible patch preserves the exact measured source hash. Full make check and make cross-lint passed. Commit 494169b8 pushed to PR 131; CI pending. Real-root experiment is honestly inconclusive for the live large tree and conservative scopes remove its speed benefit; all 16 preserved 26-hour fixture runs fail bounded historical completion. These are acceptance findings, not source test failures; remaining production work stays in fdu-uwhl.
