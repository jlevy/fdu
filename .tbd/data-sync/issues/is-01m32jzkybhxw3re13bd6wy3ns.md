---
type: is
id: is-01m32jzkybhxw3re13bd6wy3ns
title: "PR #97 review R5: DT_UNKNOWN stats counter rustdoc"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels: []
dependencies: []
parent_id: is-01m32jzamqf58124kga00kdbd0
created_at: 2026-09-21T18:17:20.075Z
updated_at: 2026-09-21T18:17:20.075Z
---
Low. Comment 5765288334. listed_child_kind_and_attrs rustdoc and H72 registry row: the skip is a no-op where d_type is DT_UNKNOWN, and the stats counter does not see the fallback stat.
