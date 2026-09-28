---
type: is
id: is-01m3ky3643py36em7htzjqmgqw
title: "timeline.py: recognize balanced-1m synthetic subject provenance"
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T11:58:36.674Z
updated_at: 2026-09-28T11:58:36.674Z
---
Flag from the 2026-09-28 macOS rerun (PR #147): the timeline's synthetic-subject check does not recognise balanced-1m provenance, so macOS and Linux runs on that generated tree are marked as real trees. Fix in the benchmark timeline tooling (timeline.py).
