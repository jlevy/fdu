---
type: is
id: is-01m74qj6rgmyg2sfkv1nvzdta0
title: "PR #209 G4: a filesystem the walk automounts is two rows, one keyed by the trigger's device"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m74qj25f6rvk77ybzaz8rszg
created_at: 2026-11-11T07:18:39.632Z
updated_at: 2026-11-11T08:42:34.108Z
closed_at: 2026-11-11T08:42:34.107Z
close_reason: "Fixed in 05029aac: a directory entered across a device is recorded from its listing's descriptor on every route; one row per automounted filesystem."
resolution: null
duplicate_of: null
---
Low. query_volumes.rs:172-181; the walk's stat of a listed child reads the trigger. Fix: key the boundary by the filesystem the listing found. Review G on PR #209: https://github.com/jlevy/fdu/pull/209#issuecomment-6789444811
