---
type: is
id: is-01kzky8bctckj3kk8gwntbg8tn
title: Make CLI tree rendering iterative and stack-safe
kind: bug
status: closed
priority: 2
version: 7
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - cli
  - correctness
dependencies:
  - type: blocks
    target: is-01kzg4bey8nn4k8y1daxc9exhd
  - type: blocks
    target: is-01kzg4bf862ajh8g2tmv5bznng
  - type: blocks
    target: is-01kzmnxy0xvkvazmqvdwsjm20h
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T18:58:38.617Z
updated_at: 2026-08-10T02:09:06.582Z
closed_at: 2026-08-10T02:09:06.581Z
close_reason: Replaced human rendering, JSON truncation detection, and JSON serialization recursion with explicit work stacks. The 1024-directory regression passes on a 64 KiB thread stack while exact human and fdu.tree/2 goldens remain unchanged.
---
Human rendering, JSON truncation detection, and JSON tree rendering recurse once per retained directory. A deep tree with a large display depth can therefore exhaust the process stack on normal input. Add a discriminating child-process test over a deep synthetic index so the current failure cannot abort the test runner, then replace all three recursive walks with explicit frames. Preserve byte-for-byte human and fdu.tree/2 output, sorting, per-directory limits, truncation semantics, streaming writes, and broken-pipe propagation under the existing golden suite. Do not solve this by silently clamping a user-requested depth unless the CLI contract explicitly adopts and tests such a bound.
