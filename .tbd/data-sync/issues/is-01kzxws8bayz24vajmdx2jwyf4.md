---
type: is
id: is-01kzxws8bayz24vajmdx2jwyf4
title: "H67: Isolate the macOS directory-open syscall floor against dumac"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
  - macos
dependencies:
  - type: blocks
    target: is-01kzy09g92seeh160bbh3m74nk
  - type: blocks
    target: is-01kzy1w2vbam0mr1z5we4y6fy0
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T15:45:19.716Z
updated_at: 2026-08-13T17:14:15.274Z
closed_at: 2026-08-13T17:04:32.544Z
close_reason: Completed the exact-binary replay and profile. Current FDU and dumac both spend about 94-96% of worker top-frame residency in synchronous open/getattrlistbulk work; the main threads wait. The published quiet-host comparison remains a statistical tie, while exact binaries show a dumac lead under heavier host pressure because dumac sustains greater concurrency. This isolates a host-sensitive kernel/concurrency floor rather than a functional or reconciliation regression. H69/fdu-hzf0 owns the bounded open-ahead follow-up.
---
The selected-total prototype removed half of FDU user CPU and much of its memory without materially changing wall time, while the definitive FDU-versus-dumac paired interval crossed zero. Establish whether any reproducible wall gap exists before changing production: profile exact current binaries side by side on the immutable near-million APFS tree, quantify per-directory open and getattrlistbulk count/time plus scheduling residue, and distinguish statistical noise from a kernel-work difference. Pre-register any candidate only from that evidence, preserve strict parsing, exact paths, fallback, scope, and partial-result semantics, and keep it only if it improves FDU wall by at least 3% with a confidence interval below zero and establishes a significant lead over dumac. Close as no-gap evidence if the tied result reproduces.
