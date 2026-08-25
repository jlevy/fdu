---
type: is
id: is-01m0wqh3ydkn7yztpt80t9d857
title: Make live one-filesystem admission fail open portably
kind: bug
status: open
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5020603690
    at: 2026-08-25T15:10:52.875Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5412701379
    at: 2026-08-25T15:25:13.078Z
  - kind: other
    url: https://github.com/jlevy/fdu/actions/runs/32865588065/job/97860038258
    at: 2026-08-25T16:06:42.801Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5413222445
    at: 2026-08-25T16:06:42.802Z
labels:
  - pr47-review
dependencies: []
parent_id: is-01kzqn502680awzhvddzntq32d
created_at: 2026-08-25T15:09:57.581Z
updated_at: 2026-08-25T16:06:54.621Z
---
At PR 47 exact head 5eb25743f9b7b1a8626bfd1998a5f5ae5bea0e10, watch::admitted maps an unavailable root device to 0, then on_root_filesystem rejects any successfully statted nonzero-device parent. This contradicts the documented fail-open behavior on transient root stat failure. Preserve absence as Option and bypass only this axis when root identity is unavailable. Exact-head CI has 18 green checks and one Windows failure at https://github.com/jlevy/fdu/actions/runs/32865588065/job/97860038258: scan::tests::the_filesystem_boundary_is_asked_of_the_parent_not_the_entry asserts a real-device distinction on Windows, where device is always 0 and one_filesystem is unsupported. Platform-gate the real-filesystem test or inject a portable device probe, while retaining a separate production test for missing-root-device fail-open behavior.
