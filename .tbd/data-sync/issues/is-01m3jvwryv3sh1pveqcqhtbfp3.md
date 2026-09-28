---
type: is
id: is-01m3jvwryv3sh1pveqcqhtbfp3
title: "Spike: real-root FSEvents cached refresh against full-scan oracle"
kind: task
status: in_progress
priority: 1
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:00:54.990Z
updated_at: 2026-09-28T02:02:02.627Z
started_at: 2026-09-28T02:02:02.625Z
---
Extend the committed standalone replay probe to read existing roots without modifying them. Capture metadata baseline and pre-scan cursor, exit, replay after real activity, reconcile dirty scopes, persist candidate independently, then compare against full metadata oracles with concurrent changes explicitly classified. Test the user-selected agent-state roots; use external scratch and retain sanitized evidence. Review prior research and modern native references. No production engine or public API change.
