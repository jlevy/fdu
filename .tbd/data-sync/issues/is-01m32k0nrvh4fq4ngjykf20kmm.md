---
type: is
id: is-01m32k0nrvh4fq4ngjykf20kmm
title: "PR #97 review R3: one bool drives H147 recycle and H72 d_type skip"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m32h6d4szbwm3n1sgxqyfn9x
created_at: 2026-09-21T18:17:54.715Z
updated_at: 2026-09-21T18:17:54.715Z
---
crates/fdu-core/src/scan.rs:1430 recycle_batches: bool is passed at :1510 as skip_dir_symlink_stat and at :2292 selects walk_worker_recycling; StreamingEmission::recycling sets skip_dir_symlink_stat true. Two independently measured keeps on one flag named for one of them. Fix: a two-variant enum (SinkMode) whose named properties both sites read. PR #97 senior review, Medium.
