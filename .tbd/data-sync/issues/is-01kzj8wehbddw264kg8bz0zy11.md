---
type: is
id: is-01kzj8wehbddw264kg8bz0zy11
title: "PR #1 review C1: Preserve known state on watcher stat errors"
kind: bug
status: closed
priority: 1
version: 3
labels:
  - pr-review
dependencies: []
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:54.090Z
updated_at: 2026-08-09T03:43:19.628Z
closed_at: 2026-08-09T03:43:19.627Z
close_reason: "Fixed: watcher verification emits Remove only for NotFound; every other stat failure emits VerificationFailed invalidation so known state is preserved for retry and error reporting during reconciliation."
---
PR #1 Cursor thread C1: https://github.com/jlevy/fdu/pull/1#discussion_r3742309820. File: crates/fdu/src/watch.rs. Only NotFound may produce Remove. Permission, transient, and other verification errors must surface or escalate without deleting indexed state.
