---
type: is
id: is-01m0rw7d4h3t49rwvk11cmk5xb
title: The bundled read carries only three projections, not the query algebra
kind: feature
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T03:15:04.465Z
updated_at: 2026-08-24T03:15:04.465Z
---
The contract's point 2 is that "every result carries the exact version, resume cursor,
lifecycle and coverage facts, and work counters that describe THE SAME observation
boundary", over a closed algebra of nine query kinds: entry, directory, filtered_tree,
rollup, navigation, recent, catalog, metadata, diagnostics.

fdu's ReadRequest carries three: children_of (directory), rollups (rollup), total. Every
other kind is reachable only through Index.report(), which takes its own guard.

So a consumer wanting a directory listing AND a recent list at one instant cannot get one.
It makes two calls, a write lands between them, and the page is internally inconsistent in
exactly the way the bundled read was built to prevent -- the rows say one thing, the
sidebar another, and both are individually true. That is the same defect fdu-2ivi fixed
for the listing-plus-header case, still present one level up.

WHAT THIS IS NOT: reproducing MetaBrowser's query types in fdu. The engine already answers
every one of these kinds; what it lacks is the ability to answer SEVERAL under one guard
and return them with one version, cursor, state and work record. The likely shape is
ReadRequest gaining the projections report() already knows how to build, evaluated inside
the same read guard, with the work record summing across them.

Sequence after fdu-jkq2 (coverage reason) so the state facts a bundle carries are the
final ones, and check the work counters still add up per projection rather than only in
total -- a bundle that hides which projection cost what is a counter that stopped being
useful.
