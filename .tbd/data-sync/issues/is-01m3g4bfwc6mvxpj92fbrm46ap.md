---
type: is
id: is-01m3g4bfwc6mvxpj92fbrm46ap
title: Make bare Makefile scratch probes honor TMPDIR on macOS
kind: bug
status: in_progress
priority: 2
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-27T00:31:02.538Z
updated_at: 2026-09-27T00:33:25.147Z
started_at: 2026-09-27T00:33:25.146Z
---
During external-scratch validation, macOS mktemp without an explicit template used the internal Darwin temporary directory despite exported TMPDIR. Pass TMPDIR-based templates for the permission preflight and performance-ledger check, matching other Makefile recipes. Verify both recipes with external scratch.
