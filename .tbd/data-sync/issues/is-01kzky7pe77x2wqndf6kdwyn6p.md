---
type: is
id: is-01kzky7pe77x2wqndf6kdwyn6p
title: Seal the minimal guard-free Rust API before first release
kind: task
status: open
priority: 1
version: 9
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
  - type: blocks
    target: is-01kzm5eqahbmtm5gwhf6fmejwh
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T18:58:17.155Z
updated_at: 2026-08-09T21:04:56.544Z
---
Inventory the actual CLI, Python, watcher, snapshot, and intended server consumers, then make every other item private or crate-private. IndexHandle must not expose std::sync::RwLockReadGuard, RwLockWriteGuard, std Receiver, or any callback/closure that can execute arbitrary user work while an internal lock is held. Replace read with focused operations that acquire and release internally and return owned values, validated results, or coherent immutable data. Since/history results must not borrow through a guard. A shared snapshot path, if supported, captures one coherent image before releasing the lock and performs serialization and filesystem I/O afterward. No filesystem I/O, blocking channel operation, Python conversion, or user sink executes under an index lock; no API sequence can self-deadlock by retaining a read capability and then applying. Preserve the single-writer linearization contract and typed poison handling without promising std RwLock as the implementation. Explicitly document Python behavior: native open/scan/refresh releases the GIL, but concurrent calls on one PyIndex are either serialized or rejected by the PyO3 borrow contract, not concurrent shared-index reads. Offer borrowed child iteration on an owned Index rather than forcing Vec allocation. Remove duplicate module/root API paths unless deliberate. After shrinking the surface, make rustdoc with missing-docs denied pass, add must-use only to semantically important values, make clock advancement checked, and correct public documentation that currently calls fixed-width CLI output width-aware. No compatibility shim is required for an unpublished API. Acceptance includes red-before-green self-deadlock/lock-release tests and the consumer inventory in docs.
