---
type: is
id: is-01m6yn4ae3k3zew55hvb5p2nqb
title: "Volumes reader: section, observation argument, unknown without one, entry-path labels, provenance tier"
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6yn4b92ezxh72j5mbh9svsp
  - type: blocks
    target: is-01m6yn4c4102x1n9x7pj3ambwp
parent_id: is-01m2xbag4cjyqqvq776rnfxcyt
created_at: 2026-11-08T22:40:40.899Z
updated_at: 2026-11-08T22:40:42.625Z
---
ViewSpec::Volumes; report and report_roots take Option<&CapacityObservation> (public signature change); without an observation capacity fields are unknown (null) with a note naming observe_capacity; rows per device the report counted, labeled by the bytewise-first path where the report entered it, with a note when there are others; counted files/bytes/allocated per row; capacity provenance tier with observed_at; never stored in snapshots.
