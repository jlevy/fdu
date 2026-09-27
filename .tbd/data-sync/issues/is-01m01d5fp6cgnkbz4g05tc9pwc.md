---
type: is
id: is-01m01d5fp6cgnkbz4g05tc9pwc
title: Make content self-check request all type rows
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-15T00:29:20.965Z
updated_at: 2026-09-27T08:12:59.854Z
closed_at: 2026-09-27T08:12:59.853Z
close_reason: "Implemented duplicate findings: scripts/content-selfcheck.mjs uses --limit all for grouped and code reports and asserts tracked TOML. Current PR #133 CI 36303716655 passes."
resolution: null
duplicate_of: null
---
scripts/content-selfcheck.mjs asserts that TOML appears in the type rows but invokes fdu with the default top-10 projection. On the current tracked archive TOML is the 11th row, so make check fails with 'missing tracked file type toml'. Adding --limit all to the self-check invocation makes the assertion match its intended complete-inventory contract. Discovered while validating the performance white-paper copy edit; keep the fix out of that documentation-only PR.
