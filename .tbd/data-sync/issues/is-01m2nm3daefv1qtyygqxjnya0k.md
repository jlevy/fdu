---
type: is
id: is-01m2nm3daefv1qtyygqxjnya0k
title: "PR #69 review 69-2: cache-design says every --no-gitignore alternation replaces the snapshot"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2nm36wpq4crkj85j40kmnn0
created_at: 2026-09-16T17:26:45.325Z
updated_at: 2026-09-16T17:37:30.108Z
closed_at: 2026-09-16T17:37:30.107Z
close_reason: "e6e38cd (PR #69): cache-design.md says an alternation replaces the snapshot only when the run retains an index; a transient-tier --no-gitignore summary replaces nothing."
resolution: null
duplicate_of: null
---
docs/project/guides/cache-design.md:42-44@fe5cb70 (PR #69). "replaces the root's one snapshot each time" is over-general: a --no-gitignore --view summary answered by the transient tier writes no snapshot (crates/fdu-core/src/lib.rs:176-178, crates/fdu-core/src/execution.rs:151-153). Narrow it to runs that retain an index.
