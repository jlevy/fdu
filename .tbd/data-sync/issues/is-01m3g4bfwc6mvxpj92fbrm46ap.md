---
type: is
id: is-01m3g4bfwc6mvxpj92fbrm46ap
title: Make bare Makefile scratch probes honor TMPDIR on macOS
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-27T00:31:02.538Z
updated_at: 2026-09-27T00:45:39.337Z
started_at: 2026-09-27T00:33:25.146Z
closed_at: 2026-09-27T00:45:39.335Z
close_reason: "PR #131 passes explicit TMPDIR-based templates to both remaining bare Makefile mktemp calls. Permission preflight and ledger check passed with external scratch; full local and cross-platform CI gates passed."
resolution: null
duplicate_of: null
---
During external-scratch validation, macOS mktemp without an explicit template used the internal Darwin temporary directory despite exported TMPDIR. Pass TMPDIR-based templates for the permission preflight and performance-ledger check, matching other Makefile recipes. Verify both recipes with external scratch.
