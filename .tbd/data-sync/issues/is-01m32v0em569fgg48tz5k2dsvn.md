---
type: is
id: is-01m32v0em569fgg48tz5k2dsvn
title: "PR #103 review R1: --format paths and --long escape backslash, doubling the Windows separator; golden matches it (Blocker)"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6ewht09ryvnt0f5zmhz5
created_at: 2026-09-21T20:37:36.004Z
updated_at: 2026-09-21T21:11:47.779Z
closed_at: 2026-09-21T21:11:47.778Z
close_reason: "Fixed in 9c22e8aa: flat_path escapes control characters only; golden uses [SEP] and fails on a doubled separator (verified 166/2 with the separator doubled, 168/0 restored)."
resolution: null
duplicate_of: null
---
R1 Blocker on PR #103 review (https://github.com/jlevy/fdu/pull/103#issuecomment-5764980291). crates/fdu-core/src/report_format.rs:170-181 flat_path() escapes backslash as well as control characters; on Windows the separator is backslash so --format paths prints paths that do not exist. tests/golden/cli-axes.tryscript.md uses [JSON_SEP] inside text output lines, so it matches the doubled separator. Fix: escape control characters only; change the golden pattern to [SEP]; verify the golden fails on a doubled separator.
