---
type: is
id: is-01kzky7pe77x2wqndf6kdwyn6p
title: Seal the minimal guard-free Rust API before first release
kind: task
status: open
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - api
  - concurrency
  - docs
dependencies:
  - type: blocks
    target: is-01kzky8gxazfdstfgbv3m9fa58
  - type: blocks
    target: is-01kzg4akvjfp8s9h0a1vs7h1c4
  - type: blocks
    target: is-01kzg49sfhtxshw3senkhjmc24
  - type: blocks
    target: is-01kzg49sswr78gpjykxctbe6c7
  - type: blocks
    target: is-01kzg4bfw0zmmztg25v9a0nkq4
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T18:58:17.155Z
updated_at: 2026-08-09T18:59:07.072Z
---
Inventory the actual CLI, Python, watcher, and intended server consumers, then make every other item private or crate-private. IndexHandle::read must stop returning std::sync::RwLockReadGuard; expose bounded operations that complete under the lock and return plain owned data or results so the internal synchronization strategy remains replaceable. Offer borrowed child iteration on an owned Index rather than forcing Vec allocation. Remove duplicate module and root API paths unless both are deliberate. After shrinking the surface, make rustdoc with missing-docs denied pass, add must-use only to semantically important values, make clock advancement checked, and correct public documentation that currently calls fixed-width CLI output width-aware. No compatibility shim is required for an unpublished API.
