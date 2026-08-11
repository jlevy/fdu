---
type: is
id: is-01kzqn1e57thy4skv9yjtcpp2h
title: "P1: formats text/json/jsonl/yaml and the fdu.report/1 schema"
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn1shjfrhncb9bhebyqx73
  - type: blocks
    target: is-01kzqn4rdq9vy4qvcve073rfhf
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:34:32.614Z
updated_at: 2026-08-11T16:01:40.602Z
closed_at: 2026-08-11T16:01:40.601Z
close_reason: "report_format.rs renders text/json/jsonl/yaml over query::Report with the fdu.report/1 schema constant and a schema-bump guard test. format_rfc3339 added to query/parse.rs as the exact inverse of parse_when (6 round-trip cases). Hand-written rather than serde: three dependency additions with the YAML half unsettled, against a small closed schema and an existing hand-written-JSON precedent; decision recorded in the module header. 10 tests including a JSON-balance check across every view."
---
CLI-feature format module over the structured Report. serde::Serialize derives enter the core crate (record in deny.toml). Renderers: text (files view prints one path per line and nothing else so it pipes into xargs; per-entry fields live in machine formats), json, jsonl, yaml. fdu.report/1 supersedes fdu.tree/2 - this spec explicitly authorizes the replacement the CLI UX plan forbade: schema, generator, root/root_raw, scan_started_at, generated_at, source, cache, complete, freshness, scope, selection, reports[]. YAML dependency: serde_yaml is unmaintained, so either a small first-party emitter over the already-structured Report or a vetted maintained crate through the 14-day cool-off; record the decision in deny.toml either way. Golden work per golden-testing-guidelines: fdu.report/1 fixture plus a schema-bump test that fails on unversioned change; tryscript goldens stay byte-stable (integers only, no floats in text) and normalize both timestamps through frontmatter patterns rather than eliding whole lines.

## Notes

Sequencing: (1) format_rfc3339 in query/parse.rs as the inverse of parse_when, round-trip tested; (2) crates/fdu/src/report_format.rs behind the cli feature rendering text/json/jsonl/yaml over query::Report, hand-written like the existing cli.rs JSON so no serde/serde_yaml cool-off is needed; (3) fdu.report/1 schema constant + schema-bump test.
