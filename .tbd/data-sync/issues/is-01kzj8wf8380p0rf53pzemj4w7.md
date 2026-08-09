---
type: is
id: is-01kzj8wf8380p0rf53pzemj4w7
title: "PR #1 review C4: Retain Python scan configuration across refresh"
kind: bug
status: closed
priority: 2
version: 2
labels:
  - pr-review
dependencies: []
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:54.819Z
updated_at: 2026-08-09T04:06:09.162Z
closed_at: 2026-08-09T04:06:09.161Z
close_reason: PyIndex stores its originating ScanConfig and applying refresh reuses it; installed-wheel smoke proves max_depth does not widen.
---
PR #1 Cursor thread C4: https://github.com/jlevy/fdu/pull/1#discussion_r3742309834. File: crates/fdu-py/src/lib.rs. Store the originating semantic ScanConfig in PyIndex and reuse it for refresh and reconciliation.
