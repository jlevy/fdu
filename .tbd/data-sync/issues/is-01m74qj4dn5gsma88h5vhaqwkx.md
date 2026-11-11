---
type: is
id: is-01m74qj4dn5gsma88h5vhaqwkx
title: "PR #209 G2: cache-only volumes joins the live root device to the snapshot's devices"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m74qj25f6rvk77ybzaz8rszg
created_at: 2026-11-11T07:18:37.237Z
updated_at: 2026-11-11T08:42:32.018Z
closed_at: 2026-11-11T08:42:32.018Z
close_reason: "Fixed in 71b026b2: an unverified root is keyed by the device its entries recorded (root_key); both reproductions honest."
resolution: null
duplicate_of: null
---
Medium. query_volumes.rs:149-155, :172-181, :266-281. Fix: on an unverified index key the root by the device its retained entries recorded; G1's check covers the other rows. Review G on PR #209: https://github.com/jlevy/fdu/pull/209#issuecomment-6789444811
