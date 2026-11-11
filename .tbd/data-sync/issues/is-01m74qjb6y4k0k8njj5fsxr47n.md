---
type: is
id: is-01m74qjb6y4k0k8njj5fsxr47n
title: "PR #209 G8: test the F2 race outcome and read the autofs token as 32 bits"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m74qj25f6rvk77ybzaz8rszg
created_at: 2026-11-11T07:18:44.190Z
updated_at: 2026-11-11T08:42:38.174Z
closed_at: 2026-11-11T08:42:38.174Z
close_reason: Fixed in cee84c82 (race outcome test) and 71b026b2 (32-bit autofs token).
resolution: null
duplicate_of: null
---
Suggestion. query_volumes.rs:352 (the filesystem was not observed) untested; tests/volumes_automount.rs:97-106 reads autofs_wqt_t as c_ulong. Review G on PR #209: https://github.com/jlevy/fdu/pull/209#issuecomment-6789444811
