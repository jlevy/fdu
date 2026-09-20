---
type: is
id: is-01m2yggj8ftk87vsp9849jb3v0
title: Fix disk cleanup skill discovery and eligibility rules
kind: chore
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-09-20T04:17:11.950Z
updated_at: 2026-09-20T04:26:20.133Z
closed_at: 2026-09-20T04:26:20.132Z
close_reason: "Updated installed personal disk cleanup skill and read-only helpers: merged clean inactive whole-worktree eligibility, independent generated-artifact eligibility, Git registry discovery, broader disk coverage. Validated with 11 regression tests, skill validation, and shell syntax checks."
resolution: null
duplicate_of: null
---
Update the installed personal skill and its audit helpers to cover all registered worktrees, application data, and temporary builds; require merged and clean inactive checkouts for whole-tree staging; report generated artifacts independently; validate using isolated Git fixtures.
