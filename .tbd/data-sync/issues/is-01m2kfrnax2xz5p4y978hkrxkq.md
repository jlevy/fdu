---
type: is
id: is-01m2kfrnax2xz5p4y978hkrxkq
title: "PR #63 review PR63-Q-1: split the control budget and the per-line guard into two independent limits"
kind: feature
status: in_progress
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-15T21:32:29.915Z
updated_at: 2026-09-15T21:41:24.599Z
---
PR #63 review PR63-Q-1 at 1fd71a9 (control.rs:235, scan.rs:3070). The user's decision (fdu-okne / fdu-1onj notes, 'DECISION, refining Q3'): ControlLimits { budget, line_limit }, defaults 4 MiB and 16 KiB, each a size or unbounded; --gitignore-budget SIZE|all and --gitignore-line-limit SIZE|all; Rust ScanConfig/OpenOptions and Python ScanOptions/OpenedOptions fields to match; both limits in ignore_rules_fingerprint; every refusal names the limit that fired and its flag; JSON ignore_rules carries both limits; snapshot parser 256 MiB ceiling independent; --help says 'all' on the budget lifts the read bound.
