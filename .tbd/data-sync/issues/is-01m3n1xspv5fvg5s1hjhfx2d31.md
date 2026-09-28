---
type: is
id: is-01m3n1xspv5fvg5s1hjhfx2d31
title: "H167: walker-assigned directory tokens replace the PathBuf-keyed directory map"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-28-pdu-and-the-linux-peer-gap.md
labels:
  - performance
  - linux
  - experiment
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
created_at: 2026-09-28T22:24:48.858Z
updated_at: 2026-09-28T22:24:48.858Z
---
Removes a SipHash map insert/remove and cross-thread PathBuf frees per directory in the detached builder (candidate from fdu-578e). Registry row in docs/project/guides/performance-loop.md; mechanism and pre-registration in the pdu brief.
