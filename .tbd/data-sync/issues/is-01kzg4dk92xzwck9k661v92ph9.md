---
type: is
id: is-01kzg4dk92xzwck9k661v92ph9
title: "fdu phase 0: repository scaffold and end-to-end vertical slice"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
created_at: 2026-08-08T07:29:24.258Z
updated_at: 2026-08-08T07:29:24.612Z
closed_at: 2026-08-08T07:29:24.612Z
close_reason: null
---
Stand up the repo with the architecture expressed in working, tested code before any optimization work.

Delivered: delta contract (types.rs); parent-pointer arena index with hierarchical reducers and O(depth) apply (index.rs); portable walk + revalidate producing deltas (scan.rs); snapshot with engine-fingerprint invalidation, atomic temp+rename, and corrupt-equals-empty (snapshot.rs); notify-backed watch layer doing coalesce -> verify-by-stat -> delta with Flag::Rescan escalated rather than dropped (watch.rs); CLI with human tree output and versioned JSON (cli.rs); PyO3 bulk-API bindings releasing the GIL (fdu-py); CI across three OSes with clippy pedantic, MSRV, docs, cargo-deny, and a wheel smoke test.

52 tests green, clippy pedantic clean with unsafe_code = deny on the core crate, --no-default-features path built and tested in CI.

Two defects found by building it, both fixed:
- The corrupt-snapshot test failed on first run: a declared entry count from the file was sizing a Vec::with_capacity, so a corrupt cache aborted the process on allocation instead of failing closed. Parsers now bound declared counts against bytes actually present.
- The watcher deadlocked on shutdown because the worker thread was joined before the notify watcher was dropped, and dropping it is what closes the channel the worker waits on. Ordering is now explicit and commented.

Explicitly NOT delivered, and tracked separately: the fast walker (fdu-atqk), the real snapshot format (fdu-xihx), and any performance claim at all.
