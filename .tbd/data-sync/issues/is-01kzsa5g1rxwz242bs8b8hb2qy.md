---
type: is
id: is-01kzsa5g1rxwz242bs8b8hb2qy
title: "PR#6 D5: provenance composition and lifecycle need a state machine, not three minima"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:03:00.151Z
updated_at: 2026-09-17T02:10:51.776Z
closed_at: 2026-09-17T02:10:51.775Z
close_reason: "Lifecycle shipped in #48 (LifecyclePhase, Coverage, CoverageReason); composition and clocks carried by fdu-fka6"
resolution: duplicate
duplicate_of: is-01kzs518dfemmfeypev5s384j8
---
plan-2026-08-11-fdu-progressive-results.md:289-318. weakest/oldest/worst are non-invertible under deletion; needs recompute algorithm and churn cost. Two per-index clocks cannot mean per-observation time. Complete|Partial insufficient for truncated/cancelled/failed/refreshing. High.
