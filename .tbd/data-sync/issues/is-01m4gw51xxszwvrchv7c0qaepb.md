---
type: is
id: is-01m4gw51xxszwvrchv7c0qaepb
title: "PR #192 A3: Overlap check misses a root that is an alias inside another root"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4gw4em1dbpcwbmgqzw911k9
created_at: 2026-10-09T17:42:39.292Z
updated_at: 2026-10-09T21:28:54.413Z
closed_at: 2026-10-09T21:28:54.413Z
close_reason: "fixed on #192 (b7f72218..35266cab); dispositions: https://github.com/jlevy/fdu/pull/192#issuecomment-6089558340"
resolution: null
duplicate_of: null
---
Severity: Medium. PR #192 review A (https://github.com/jlevy/fdu/pull/192#issuecomment-6085921156), finding A3. Where: crates/fdu-core/src/query/query_request.rs:434-438; docs/usage.md:86-88; CHANGELOG.md:33-34.
