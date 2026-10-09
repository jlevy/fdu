---
type: is
id: is-01m4gw5axm75ggmd7sq7xbem87
title: "PR #192 C4: Roots::resolve is quadratic, stats ancestors for one root, runs under the GIL and before the ticker"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4gw4em1dbpcwbmgqzw911k9
created_at: 2026-10-09T17:42:48.498Z
updated_at: 2026-10-09T21:28:56.983Z
closed_at: 2026-10-09T21:28:56.983Z
close_reason: "fixed on #192 (b7f72218..fa62a7b1); dispositions: https://github.com/jlevy/fdu/pull/192#issuecomment-6089558882"
resolution: null
duplicate_of: null
---
Severity: Medium. PR #192 review C (https://github.com/jlevy/fdu/pull/192#issuecomment-6085911373), finding C4. Where: query_request.rs:455, 408; fdu-py lib.rs:1077; cli.rs:860.
