---
type: is
id: is-01m3r6z458hp6q8vk6qfqj60yx
title: Decide the snapshot write's durability policy (F_FULLFSYNC / fdatasync / none)
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-30T03:50:38.504Z
updated_at: 2026-09-30T03:50:38.504Z
---
Split out of fdu-n75m part 3, which needs a person. The one-shot snapshot writer runs sync_all (F_FULLFSYNC on Apple) before renaming a checksummed, corrupt-equals-empty cache file whose reader fails closed on a torn write; a repeated run spends ~41 ms after its last byte (exp-068 measurement) and most of that is the fsync. fdatasync semantics or no sync at all are each defensible for a cache that is verified on read, and the choice is a durability policy rather than an engineering fact. Decide, then measure on the default-path job per docs/project/guides/performance-loop.md. Parts 1 (flush before join, exp-068/H101) and 2 (large-index teardown on a detached thread, exp-160, lib.rs release_index) of fdu-n75m are done.
