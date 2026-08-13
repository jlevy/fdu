---
type: is
id: is-01kzxws8bayz24vajmdx2jwyf4
title: "H67: Isolate the macOS directory-open syscall floor against dumac"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
  - macos
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T15:45:19.716Z
updated_at: 2026-08-13T15:45:19.716Z
---
The selected-total prototype removed half of FDU user CPU and much of its memory without materially changing wall time, while the definitive FDU-versus-dumac paired interval crossed zero. Establish whether any reproducible wall gap exists before changing production: profile exact current binaries side by side on the immutable near-million APFS tree, quantify per-directory open and getattrlistbulk count/time plus scheduling residue, and distinguish statistical noise from a kernel-work difference. Pre-register any candidate only from that evidence, preserve strict parsing, exact paths, fallback, scope, and partial-result semantics, and keep it only if it improves FDU wall by at least 3% with a confidence interval below zero and establishes a significant lead over dumac. Close as no-gap evidence if the tied result reproduces.
