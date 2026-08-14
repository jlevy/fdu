---
type: is
id: is-01kzz29xkn8rqb5p7367jpwb2h
title: Extension interner never reclaims ids, so a long-lived index grows without bound
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:03.093Z
updated_at: 2026-08-14T02:41:03.093Z
---
crates/fdu/src/index.rs intern_ext only ever appends to ext_names and ext_ids; nothing releases an entry when the last file using an extension is removed. remove_subtree frees entries without touching the interner.

For a one-shot CLI scan this is harmless. For fdu --watch, which is the long-lived case the crate is designed around, a tree that churns through distinct extensions (editor temp files, build artifacts, hashed asset names) grows both maps without bound for the life of the process.

PR #4 added per-id refcounts and a free list so an id and its name are reclaimed when the last referencing file goes away, plus a churn regression test. Port it, adapted to main's index which now also tracks allocated bytes per ExtTally.
