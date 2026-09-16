---
type: is
id: is-01m2mcqtkydqgn6smqjcdeczkj
title: "PR #67 delta D67-2: the orphan rule's comment states the opposite of the deletion it guards"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m2mckkzdt9pgwx782pcr49r5
created_at: 2026-09-16T05:58:51.261Z
updated_at: 2026-09-16T06:31:31.041Z
closed_at: 2026-09-16T06:31:31.040Z
close_reason: "02b44ee: the orphan rule's comment now states the rule the deletion follows, that a sidecar is kept only while a snapshot image is at its snapshot path"
resolution: null
duplicate_of: null
---
Delta review 5218953084 of PR #67, P3, REPRODUCED by the reviewer.

`crates/fdu-core/src/cache.rs:551-554@a5e2124`.

The comment says a sidecar whose snapshot path holds an unrecognized file "is not
orphaned" and stays; `is_snapshot_image` is false for a foreign file, so the code removes
it. The code is the defensible answer; the comment states the opposite rule on the line
that decides the deletion.

Fix: correct the comment.
