---
type: is
id: is-01m2nm7wv38xdhz976g7vazm9b
title: "PR #71 review 71-2: streaming plan's remaining-beads list omits fdu-pro1"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2nm7m7kc7hna3r0cr3hyxf3
created_at: 2026-09-16T17:29:12.289Z
updated_at: 2026-09-16T17:37:33.409Z
closed_at: 2026-09-16T17:37:33.408Z
close_reason: "1dbd5fd (PR #71): the streaming plan lists fdu-pro1 as in progress, open until a quiet-host parity run against b75bf85 proves parity; neither fixed nor a blocker."
resolution: null
duplicate_of: null
---
docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md:1140-1141 @4c70e10 (PR #71). The "what remains" annotation under closing the linked beads omits fdu-pro1 (P0, in progress, named in the plan's Bead Graph section), which records the 3.6-10x whole-scan regression found on the rewrite branch and stays open until a quiet-host parity run against the pre-rewrite control proves parity. Add it with that acceptance condition; do not describe it as fixed or as a blocker in either direction, since its acceptance measurement has not run.
