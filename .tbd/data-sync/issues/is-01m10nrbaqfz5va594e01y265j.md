---
type: is
id: is-01m10nrbaqfz5va594e01y265j
title: Reconcile MetaBrowser change relay, root replacement, and shutdown boundaries
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nrbpdhjb0srvkkhs7mwx7
parent_id: is-01m0y1sgqd1sd33stssgw25f2q
created_at: 2026-08-27T03:55:55.094Z
updated_at: 2026-08-27T07:56:14.422Z
closed_at: 2026-08-27T07:56:14.421Z
close_reason: "Completed across MetaBrowser commits 9cf1d87, 6ae1468, and 45266a8: provider gap/reset semantics, one provider iterator with coordinator fan-out, root replacement, and shutdown remain covered by lifecycle and race tests."
resolution: null
duplicate_of: null
---
Update coordinator change relay/merge/publish functions and _replace_root_locked/_stop_handle_locked/close. Keep provider consumer reset, provider observation gap, and host SSE replay loss distinct; discard old continuations and join the old handle before publishing the new host generation; reject stale old-root publication.
