---
type: is
id: is-01m2ngg50xq3zdxqa5vxvyty91
title: "PR #64 review RN64-7: OpenedWorkerPanicked is Rust-only; Python raises OpenedIndexError"
kind: bug
status: in_progress
priority: 3
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2ngfd0y2yzwg2v10p2j601z
hold: null
hold_until: null
created_at: 2026-09-16T16:23:48.508Z
updated_at: 2026-09-16T16:25:04.645Z
started_at: 2026-09-16T16:25:04.643Z
---
CHANGELOG.md:195@d303dc1. crates/fdu-py/src/opened_binding.rs:60-67 maps Error::OpenedWorkerPanicked to OpenedIndexError.
