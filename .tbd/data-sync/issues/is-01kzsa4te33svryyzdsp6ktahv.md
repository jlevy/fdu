---
type: is
id: is-01kzsa4te33svryyzdsp6ktahv
title: "PR#6 C2: Status::Partial used for values that may shrink"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:38.018Z
updated_at: 2026-08-11T21:02:38.018Z
---
crates/fdu/src/types.rs:171-185, index.rs:1096-1107. Partial documented as monotone lower bound but status_of maps Reconciling/Stale/error to it. Cached deletion during reconcile makes totals shrink. High.
