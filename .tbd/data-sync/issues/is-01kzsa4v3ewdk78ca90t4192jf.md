---
type: is
id: is-01kzsa4v3ewdk78ca90t4192jf
title: "PR#6 C5: provenance changes bypass the clocked delta/change-feed contract"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:38.701Z
updated_at: 2026-08-11T21:02:38.701Z
---
types.rs:1-16, index.rs:602-689,1302-1313,732-760. Unchanged upsert mutates entry.source then returns false; finish_reconcile mutates verified directly. Neither advances Clock nor reaches AppliedDelta. Medium.
