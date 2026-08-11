---
type: is
id: is-01kzq1vqasq5aqd5g07357h76t
title: "Phase 1: Query/Report core — views, selection, formats, CLI axes rework"
kind: feature
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq1vz569z3fbz6a82kat2rd
  - type: blocks
    target: is-01kzq1vzdeychrseqy1t2qftr9
  - type: blocks
    target: is-01kzq1w4rnhr2z0eamhsy19h6m
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-10T23:59:22.456Z
updated_at: 2026-08-11T00:05:33.735Z
---
query module (Selection, ViewSpec tree/types/files/summary, Query, Report, pure report()); largest/recent are compositions (files + --sort/--limit), not views. Shared value grammars parse_when (durations like 2h/1h30m or RFC 3339) and parse_size in the library; --modified-since/--modified-before half-open window; scan_started_at/generated_at stamped on every report (the sync-watermark surface). ExtTally.allocated; serde Report + text/json/jsonl/yaml formatters; fdu.report/1 schema + goldens; five-axis CLI flag rework (replaces --by-type/--json/--apparent-size/--number/--max-depth/--no-cache; 'all' for unbounded, --depth 0 keeps du semantics); SKILL/help/README/benchmark-manifest updates; Python Index.report() accepting the same string grammars.
