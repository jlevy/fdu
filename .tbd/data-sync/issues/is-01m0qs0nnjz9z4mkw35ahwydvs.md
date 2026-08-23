---
type: is
id: is-01m0qs0nnjz9z4mkw35ahwydvs
title: "Scripted watch events: a deterministic backend seam for the InvalidateReason cases"
kind: feature
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0qs19pg77zfmd3s2kg7k905
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T16:59:43.666Z
updated_at: 2026-08-23T22:12:09.011Z
closed_at: 2026-08-23T22:12:09.010Z
close_reason: "WatchBackend::Scripted reads a line-oriented event file that replaces the notify backend and nothing else — same coalescing, same stat verification, same delta path. End-to-end tests now pin two contracts nothing exercised before: a dropped-event flag escalates the subtree the backend named, and an unpaired rename escalates to the root because there is no safe bound on where the counterpart landed. Both assertions were written backwards first and corrected to what the engine actually guarantees. A third test pins the seam's safety property: a scripted create for a file that does not exist becomes a Remove, so a script can claim something may have changed but cannot state a fact the filesystem denies. Format is tab-separated lines rather than JSONL: the engine has no JSON parser and a test seam is a poor reason to acquire one."
resolution: null
duplicate_of: null
---
Causal sequencing (tests/golden/bin/watch-capture.mjs) covers changes a test can cause. It cannot cover the conditions that matter most and occur least: the whole InvalidateReason enum (engine_contract.rs:320) — WatchOverflow, UnpairedRename, WatchSetupRace, VerificationFailed, WatchContention — exists for situations a test cannot reliably provoke on a real filesystem. Add a scripted event source behind the watch feature gate: a JSONL file of backend events replaces the notify backend, flowing through the SAME coalescing, the SAME stat verification, and the SAME delta path. The seam is the backend, not the observation, so a scripted event is still verified against the real filesystem before becoming an Op — 'a watch sample is valid at its stat point' is preserved and this stays a test seam rather than a back door. One real end-to-end golden continues to cover the backend binding. This is the golden-testing guideline's 'provide a mock mode for all nondeterminism' applied at the one seam where the engine has nondeterminism it does not own.
