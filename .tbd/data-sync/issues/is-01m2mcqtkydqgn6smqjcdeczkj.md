---
type: is
id: is-01m2mcqtkydqgn6smqjcdeczkj
title: "PR #67 delta D67-2: the orphan rule's comment states the opposite of the deletion it guards"
kind: bug
status: open
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2mckkzdt9pgwx782pcr49r5
created_at: 2026-09-16T05:58:51.261Z
updated_at: 2026-09-16T05:59:01.801Z
---
Delta review 5218953084 of PR #67, P3, REPRODUCED by the reviewer.

`crates/fdu-core/src/cache.rs:551-554@a5e2124`.

The comment says a sidecar whose snapshot path holds an unrecognized file "is not
orphaned" and stays; `is_snapshot_image` is false for a foreign file, so the code removes
it. The code is the defensible answer; the comment states the opposite rule on the line
that decides the deletion.

Fix: correct the comment.
