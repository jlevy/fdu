---
type: is
id: is-01m2eefb7t5e7nxf2hdx13tfjc
title: "PR #52 review BUILD-2: a kind race during discovery permanently fails an opened root"
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-13T22:33:43.929Z
updated_at: 2026-09-13T23:40:45.528Z
closed_at: 2026-09-13T23:40:45.523Z
close_reason: "Fixed in a80696b on PR #52 with option 1: on a kind conflict prepare_scanner_batch keeps checking the scanner input contract, then prepares that batch for the general lane (exact ancestry plus kind replacement); apply_scanner_baseline dispatches through reduce_prepared. Review proof test adopted; the old atomicity test now covers the refused and the accepted replacement. Same verification status as fdu-opte: fdu-core check and clippy pass locally, tests not yet executed, CI blocked by the base conflict."
resolution: null
duplicate_of: null
---
PR #52 review BUILD-2 (High). crates/fdu-core/src/index.rs:3195-3206 and opened.rs:430-440 at afbb2ee. Refresh is supported while an opened root discovers. If p/d is listed as a directory, then replaced on disk by a file that refresh(['p/d']) inserts through the general lane, discovery's pending batch reaches prepare_scanner_batch, which returns UnsupportedScanConfig('scanner discovery cannot replace entry kinds'). run_discovery propagates it, the worker publishes DiscoveryTransition::Failed, and close() yields OpenedWorkerFailed. The base committed discovery through the general lane, where upsert_beneath replaced the kind. Fix (option 1 of 2, per the fixer brief): on a kind conflict, prepare that batch for the general lane (exact preflight plus kind replacement), paid only in the rare case. Keep the change inside the scanner preflight so it composes with #48's LIFE-1 fix. Adopt the review's proof test; scanner_parent_proof_rejects_kind_replacement_atomically pins the old rejection and changes with it.
