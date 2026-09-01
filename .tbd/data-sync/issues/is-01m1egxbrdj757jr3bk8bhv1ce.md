---
type: is
id: is-01m1egxbrdj757jr3bk8bhv1ce
title: Fuse detached control-free scanner preparation and reduction
kind: task
status: in_progress
priority: 0
version: 2
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
updated_at: 2026-09-01T13:00:44.041Z
started_at: 2026-09-01T13:00:44.040Z
---
Pre-registered H101 composite. For a detached control-free index still under construction, consume trusted ScannerBatch input in one pass: resolve each parent and mutate immediately, so preparation is eliminated rather than shifted; keep current prepared/conditional paths for public scan, opened discovery, refresh, watch, and arbitrary mutation. Combine only as one experiment with exp-084 compact control-free fixed-partition storage so final allocation/reallocation/requested-byte ratios versus b75bf85 can be <=1.05. Primary gate: metabrowser-current default-tree >=3% faster than instrumented c7b2120 with paired CI below zero. Secondary: cold-scan-index same direction; exact digests and control semantics; phase counters prove preparation removed; public scan/opened exact gates and <=3% noninferiority; complete composite reverts on any failure.
