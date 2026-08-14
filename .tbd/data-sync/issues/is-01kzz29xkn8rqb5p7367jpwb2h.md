---
type: is
id: is-01kzz29xkn8rqb5p7367jpwb2h
title: Extension interner never reclaims ids, so a long-lived index grows without bound
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:03.093Z
updated_at: 2026-08-14T02:51:23.820Z
closed_at: 2026-08-14T02:51:23.820Z
close_reason: "Added refcounts and a free list to the extension interner: intern_ext retains, remove_subtree releases through the new release_ext, and an id plus its name are reclaimed when the last referencing file goes. ext_names now holds Option<String> so a vacant slot is reissued rather than appended. Regression test the_extension_interner_reclaims_ids_after_churn verified red before the fix; a_reclaimed_extension_id_does_not_alias_a_live_tally guards reissue safety. All 350 lib tests pass."
---
crates/fdu/src/index.rs intern_ext only ever appends to ext_names and ext_ids; nothing releases an entry when the last file using an extension is removed. remove_subtree frees entries without touching the interner.

For a one-shot CLI scan this is harmless. For fdu --watch, which is the long-lived case the crate is designed around, a tree that churns through distinct extensions (editor temp files, build artifacts, hashed asset names) grows both maps without bound for the life of the process.

PR #4 added per-id refcounts and a free list so an id and its name are reclaimed when the last referencing file goes away, plus a churn regression test. Port it, adapted to main's index which now also tracks allocated bytes per ExtTally.
