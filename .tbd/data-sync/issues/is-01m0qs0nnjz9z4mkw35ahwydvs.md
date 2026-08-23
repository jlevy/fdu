---
type: is
id: is-01m0qs0nnjz9z4mkw35ahwydvs
title: "Scripted watch events: a deterministic backend seam for the InvalidateReason cases"
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0qs19pg77zfmd3s2kg7k905
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T16:59:43.666Z
updated_at: 2026-08-23T17:00:19.778Z
---
Causal sequencing (tests/golden/bin/watch-capture.mjs) covers changes a test can cause. It cannot cover the conditions that matter most and occur least: the whole InvalidateReason enum (engine_contract.rs:320) — WatchOverflow, UnpairedRename, WatchSetupRace, VerificationFailed, WatchContention — exists for situations a test cannot reliably provoke on a real filesystem. Add a scripted event source behind the watch feature gate: a JSONL file of backend events replaces the notify backend, flowing through the SAME coalescing, the SAME stat verification, and the SAME delta path. The seam is the backend, not the observation, so a scripted event is still verified against the real filesystem before becoming an Op — 'a watch sample is valid at its stat point' is preserved and this stays a test seam rather than a back door. One real end-to-end golden continues to cover the backend binding. This is the golden-testing guideline's 'provide a mock mode for all nondeterminism' applied at the one seam where the engine has nondeterminism it does not own.
