---
type: is
id: is-01m3jvwryv3sh1pveqcqhtbfp3
title: "Spike: real-root FSEvents cached refresh against full-scan oracle"
kind: task
status: in_progress
priority: 1
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:00:54.990Z
updated_at: 2026-09-28T02:21:53.211Z
started_at: 2026-09-28T02:02:02.625Z
---
Extend the committed standalone replay probe to read existing roots without modifying them. Capture metadata baseline and pre-scan cursor, exit, replay after real activity, reconcile dirty scopes, persist candidate independently, then compare against full metadata oracles with concurrent changes explicitly classified. Test the user-selected agent-state roots; use external scratch and retain sanitized evidence. Review prior research and modern native references. No production engine or public API change.

## Notes

Implementation and parent review complete: six focused self-tests, Ruff and BasedPyright pass; original four tests pass. Design assessment: isolated standard-library research instrument reuses the native helper, not shipped engine behavior; fd-relative no-follow traversal, private external state, failure preservation, explicit concurrency and resource bounds reviewed. Timing and private-state concerns fixed before source freeze. No remaining blocking source finding. Real-root and next-day outcomes documented honestly; production normalization/completion remain under fdu-uwhl. Full make check running, cross-lint passed.
