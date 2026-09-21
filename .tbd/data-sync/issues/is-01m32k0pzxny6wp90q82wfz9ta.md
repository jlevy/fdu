---
type: is
id: is-01m32k0pzxny6wp90q82wfz9ta
title: "PR #97 review R5: stats counter blind to the DT_UNKNOWN fallback stat"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6d4szbwm3n1sgxqyfn9x
created_at: 2026-09-21T18:17:55.965Z
updated_at: 2026-09-21T18:38:11.428Z
closed_at: 2026-09-21T18:38:11.428Z
close_reason: "Fixed: DT_UNKNOWN no-op and stats-counter blindness stated in listed_child_kind_and_attrs rustdoc and the H72 registry row."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/scan.rs:1224-1243 listed_child_kind_and_attrs. Where d_type is DT_UNKNOWN std's file_type() performs the non-following stat itself, bypassing metadata_for_fingerprint's c.stats += 1, so the H72 counter reads as if the skip fired. Fix: one sentence in the rustdoc and the H72 registry row (performance-loop.md). PR #97 senior review, Low.
