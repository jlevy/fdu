---
type: is
id: is-01m2ks0bb1baryjxmhbfpn6twc
title: "PR #67 review PR67-3: fdu's own leftovers are called foreign files and nothing reclaims them"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2krzt6endqrw5gq2phas5g6
created_at: 2026-09-16T00:13:59.008Z
updated_at: 2026-09-16T03:49:54.990Z
closed_at: 2026-09-16T03:49:54.988Z
close_reason: "f39b701: CacheState::Leftover(LeftoverKind::{StagingTemporary,OrphanedContent}) names fdu's own debris in text, JSON, JSONL, YAML and Python; --cache-clear=all reclaims it under the reviewed rules (name shape AND magic; STALE_TEMP_AGE reused for staging files; a sidecar only while no snapshot claims it, decided after snapshots are removed; links and directories never). clear_all_caches returns ClearSummary. Deterministic tests set mtimes outright, in Rust and in the golden's plant script."
resolution: null
duplicate_of: null
---
crates/fdu/src/cli.rs:962,991@816fcf7; tests/parity/py/parity_cli.py:310-326; crates/fdu-core/src/snapshot.rs:996,1047. A leaked staging temporary (.{hash}.fdu.tmp.*, snapshot magic) and an orphaned {hash}.fdu.content sidecar are reported unrecognized and clear says they are not fdu snapshots. DECISION (user): fix the classification so status names them fdu leftovers. Clearing may remove them only when: the name matches fdu's pattern AND the content magic matches; a staging temporary is older than the existing reaper threshold (reuse STALE_TEMP_AGE); an orphaned sidecar has no snapshot for its hash after the clear, or the clear removes that snapshot; symlinks and directories are never removed. Deterministic tests per rule, mtimes set explicitly, never sleep. If it grows beyond a contained change, keep the classification and file the reclamation as a stack-followup bead.
