---
type: is
id: is-01m32k0qk5mdfh3vbby8zhaab7
title: "PR #97 review R6: send_full leaves an unused full-capacity vec after the final flush"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6d4szbwm3n1sgxqyfn9x
created_at: 2026-09-21T18:17:56.581Z
updated_at: 2026-09-21T18:38:11.937Z
closed_at: 2026-09-21T18:38:11.937Z
close_reason: "Fixed: finish sends the last batch without allocating a successor; retained-path next_vec returns Vec::new() (pre-H147 shape); recycling path unchanged (measured with with_capacity fallback)."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/scan.rs StreamingEmission::finish -> send_full -> next_vec leaves a never-used Vec::with_capacity(1024) per worker per walk; on the public non-recycling scan path every batch after the first now starts at full capacity where mem::take left an empty vec, unmeasured. Fix: finish takes the batch without a replacement; non-recycling next_vec returns Vec::new() (the pre-PR shape). PR #97 senior review, Low.
