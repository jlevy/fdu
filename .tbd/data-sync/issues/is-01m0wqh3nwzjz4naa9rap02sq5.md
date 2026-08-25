---
type: is
id: is-01m0wqh3nwzjz4naa9rap02sq5
title: Clock cap-refused upserts that mutate existing index state
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5020603690
    at: 2026-08-25T15:10:51.574Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5412701379
    at: 2026-08-25T15:25:12.692Z
labels:
  - pr47-review
  - metabrowser
dependencies: []
parent_id: is-01m0vx6yw0f8bddcwggvk2ha0p
created_at: 2026-08-25T15:09:57.307Z
updated_at: 2026-08-25T15:25:12.693Z
---
At PR 47 exact head 353d48f6c795b72e1c4c94ed8f95b8e08b815c9b, Index::upsert_beneath mutates before the new max_files refusal: apply_upsert may create placeholder ancestor directories, and a kind-changing existing entry is removed before the cap check. The cap branch then increments refused and returns false. apply_validated consequently omits the op from the journal and, once coverage/run facts are already partial, can return without advancing the data clock even though rows or directory rollups changed. Preflight refusal before mutation or return an outcome that separately records mutation/refusal and an accurate effective delta; test a full capped index receiving a nested new file with missing parents and a directory/symlink replaced by a file, including second and later refusals, WatchBatch dirty signals, exact terminal state clock, and conservation.
