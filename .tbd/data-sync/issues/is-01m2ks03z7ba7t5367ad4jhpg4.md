---
type: is
id: is-01m2ks03z7ba7t5367ad4jhpg4
title: "PR #67 review PR67-2: docs promise non-regular files are listed unrecognized, but list_caches skips directories"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2krzt6endqrw5gq2phas5g6
created_at: 2026-09-16T00:13:51.462Z
updated_at: 2026-09-16T03:49:49.075Z
closed_at: 2026-09-16T03:49:49.073Z
close_reason: "f39b701: list_caches no longer skips directories; a directory is listed as unrecognized from its own metadata, matching the docs table, the Unrecognized doc comment and single-path status. Never descended into, never removed. Test: a_directory_in_the_cache_is_listed_and_never_removed."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/cache.rs:269@816fcf7; docs/project/guides/cache-design.md:60; cache.rs:76-78. list_caches skips directories, so one in the cache directory appears in no listing and in no Left in place count, while status for a single path reports a directory unrecognized. DECISION (user): make the docs and doc comments match the code; reporting a directory as unrecognized is also acceptable. Pick whichever keeps the listing honest and justify it in the reply.
