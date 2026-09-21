---
type: is
id: is-01m32jzkbbfh039yejepxbjbtz
title: "PR #97 review R3: name transient-fold sink mode"
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels: []
dependencies: []
parent_id: is-01m32jzamqf58124kga00kdbd0
created_at: 2026-09-21T18:17:19.467Z
updated_at: 2026-09-21T18:17:19.467Z
---
Medium. Comment 5765288334. recycle_batches bool drives both H147 recycle and H72 skip_dir_symlink_stat. Suggested SinkMode enum. Engine was approved; this is a naming refactor that expands the engine surface. Defer unless a tiny honest rename is clearly safer than leaving the bool.
