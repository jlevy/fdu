---
type: is
id: is-01m2h6p990etkx2x0vz040acj0
title: "PR #55 review PR55-INV-1: no stated container for the working inventory and replay cursor"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2h6nn916177k0wrc92zmkge
created_at: 2026-09-15T00:15:25.982Z
updated_at: 2026-09-15T00:32:13.940Z
closed_at: 2026-09-15T00:32:13.939Z
close_reason: "a59b089: working inventory and replay cursor live in the root's snapshot (derived state, VERSION + FAIL FAST, one publication boundary); checkpoints in the store; upgrade drops inventory and cursor (G2 full scan) but keeps checkpoints comparable; complete-only rule restated for typed gaps; FSEvents plan G2, format section and non-goals agree"
resolution: null
duplicate_of: null
---
PR #55, delta review 5204152578. Plan @4727de0 :222-225, :233, :294-297 and FSEvents plan :70-72, :323, :353-365. snapshot::save refuses any non-Complete coverage (snapshot.rs:189-193 at dda7e6a) and engine_fingerprint mixes the crate version (snapshot.rs:173-185), so every release discards the snapshot. Slice 3 needs a gap-bearing, cursor-bearing working inventory. Fix: choose the container consistent with the compatibility contract, rewrite the complete-only sentence, make the FSEvents plan agree, and state that an upgrade drops inventory and cursor but keeps checkpoints.
