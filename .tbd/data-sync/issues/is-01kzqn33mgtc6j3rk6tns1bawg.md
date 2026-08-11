---
type: is
id: is-01kzqn33mgtc6j3rk6tns1bawg
title: "P2: background snapshot save overlapped with rendering"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:35:27.375Z
updated_at: 2026-08-11T16:47:16.271Z
closed_at: 2026-08-11T16:47:16.270Z
close_reason: open_with_pending_save returns a joinable PendingSave; blocking open() joins it. CLI renders while the write runs and joins before raising a render error, so a broken pipe cannot abandon a finished scan's snapshot. Drop also joins. Failed save warns on stderr without changing the exit code. Only complete scans are written, with a Unix test proving a partial scan leaves the previous complete snapshot untouched. 4 tests.
---
Write ordering and failure semantics, as rules rather than heuristics. The core snapshot is written by auto and refresh only when the scan is complete and the index is Fresh (existing invariant), on a background thread overlapped with rendering: once producers finish the index is read-only, so serialization and rendering are two concurrent readers. The save must never delay first output; the process joins the save thread before exit so a write is never abandoned; the save still completes when rendering ends early, because a broken pipe must not discard a finished scan's work. A failed save (read-only cache dir, quota) is a stderr warning and never changes the exit code. read-only policy suppresses the write entirely. The write happens on every platform for every tier of query including pure stat rollups - the frontier research rejects the 'stat-only runs skip the write' refinement, since the write is tens of ms off the hot path while the stat-tier snapshot is exactly what the two decisive warm paths consume. Tests: a broken-pipe run still leaves a valid snapshot; a read-only cache dir warns and exits 0.
