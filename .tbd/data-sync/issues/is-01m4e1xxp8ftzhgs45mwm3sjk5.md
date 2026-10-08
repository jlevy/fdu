---
type: is
id: is-01m4e1xxp8ftzhgs45mwm3sjk5
title: "Performance index Phase 2: time the Linux cells and project the full index"
kind: task
status: open
priority: 2
version: 1
labels:
  - performance
  - linux
dependencies: []
created_at: 2026-10-08T15:25:53.735Z
updated_at: 2026-10-08T15:25:53.735Z
---
Phase 2 of docs/project/specs/active/plan-2026-10-05-fdu-performance-index.md: hand the same 13 milestone builds, explorations/benchmarks/index-suite.json (v1) and the history driver (explorations/benchmarks/realtree/history.py) to a Linux host; time one cell per job (cold cache, default tree, summary, code, documents, both warm-content, multi-view, both warm-metadata jobs, both opened-root jobs on K; scale on G) interleaved against v0.3.0; commit them under docs/project/reports/performance-evidence/history/ and regenerate the page so the full index (both platforms, 50% each) replaces the macOS-only score. Quote-grade cells need quiet or controlled-interactive, 20 rounds, and a non-exploratory --stage (perf_index.evidence_regime).
