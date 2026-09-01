---
type: is
id: is-01m1egxbrdj757jr3bk8bhv1ce
title: Fuse detached control-free scanner preparation and reduction
kind: task
status: closed
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01m1dtr903vj783j9ajaxfnczf
hold: null
hold_until: null
created_at: 2026-09-01T13:00:38.540Z
updated_at: 2026-09-01T13:32:23.148Z
started_at: 2026-09-01T13:00:44.040Z
closed_at: 2026-09-01T13:32:23.131Z
close_reason: H104 was implemented and measured, but default-tree improved only 1.11% with CI [-2.47%, +0.40%] and cold-scan-index was flat. exp-087 records the result; the complete composite was removed under its preregistered gate.
resolution: canceled
duplicate_of: null
---
Pre-registered H104 composite. For a detached control-free index still under construction, consume trusted ScannerBatch input in one pass: resolve each parent and mutate immediately, so preparation is eliminated rather than shifted; keep current prepared/conditional paths for public scan, opened discovery, refresh, watch, and arbitrary mutation. Combine only as one experiment with exp-084 compact control-free fixed-partition storage so final allocation/reallocation/requested-byte ratios versus b75bf85 can be <=1.05. Primary gate: metabrowser-current default-tree >=3% faster than instrumented c7b2120 with paired CI below zero. Secondary: cold-scan-index same direction; exact digests and control semantics; phase counters prove preparation removed; public scan/opened exact gates and <=3% noninferiority; complete composite reverts on any failure.
