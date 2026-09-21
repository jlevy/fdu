---
type: is
id: is-01m32jzkbbfh039yejepxbjbtz
title: "PR #97 review R3: name transient-fold sink mode"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels: []
dependencies: []
parent_id: is-01m32jzamqf58124kga00kdbd0
created_at: 2026-09-21T18:17:19.467Z
updated_at: 2026-09-21T18:54:23.456Z
closed_at: 2026-09-21T18:54:23.456Z
close_reason: "Fixed in ac891f7a on cursor/linux-perf-iterate-de1b: private enum SinkMode { Retained, TransientFold } with recycles_batches() and skips_dir_symlink_stat(); StreamingEmission::for_sink reads both; serial walker passes the named property; worker chosen by mode. Behaviour unchanged; 686 lib tests, clippy -D warnings, cross-lint clean. The type is private, so the engine surface is unchanged."
resolution: null
duplicate_of: null
---
Deferred: engine was approved; SinkMode enum expands the engine surface after that verdict. Track for a follow-up on this layer if a future measurement needs to toggle recycle and d_type skip independently.
