---
type: is
id: is-01kzy2ewdvwzgfseqa531rvwvv
title: Warm open of an unchanged tree must not clone and rewrite the snapshot
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - perf
  - linux
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-13T17:24:31.290Z
updated_at: 2026-08-13T18:11:53.497Z
---
lib.rs open_with_pending_save: after a warm reconcile, spawn_save runs unconditionally, so every warm open of a quiet tree deep-clones the entire index (peak RSS roughly doubles; 411 MiB vs 285 MiB at 450k entries on the Linux review rig) and serializes+rewrites a byte-equivalent snapshot. ApplyStats already distinguishes effective from unchanged work: when the index was loaded from a compatible snapshot and reconciliation applied zero effective deltas, return PendingSave::none(). Verify with a warm-open RSS assertion and a snapshot-mtime-unchanged test. Found during PR #8 senior review on Linux.
