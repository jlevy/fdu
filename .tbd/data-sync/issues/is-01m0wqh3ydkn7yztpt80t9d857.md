---
type: is
id: is-01m0wqh3ydkn7yztpt80t9d857
title: Make live one-filesystem admission fail open portably
kind: bug
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5020603690
    at: 2026-08-25T15:10:52.875Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5412701379
    at: 2026-08-25T15:25:13.078Z
labels:
  - pr47-review
dependencies: []
parent_id: is-01kzqn502680awzhvddzntq32d
created_at: 2026-08-25T15:09:57.581Z
updated_at: 2026-08-25T15:25:13.078Z
---
At PR 47 exact head 353d48f6c795b72e1c4c94ed8f95b8e08b815c9b, watch::admitted maps an unavailable root device to 0, then on_root_filesystem rejects any successfully statted nonzero-device parent. This contradicts the documented fail-open behavior on transient root stat failure. Preserve absence as Option and bypass only this axis when root identity is unavailable. The new unit test also runs on Windows even though one_filesystem is unsupported there and attrs.dev is always 0, causing the exact-head Windows CI failure; gate the real-device test to supported platforms or inject a portable device probe.
