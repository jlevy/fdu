---
type: is
id: is-01kzqn1e57thy4skv9yjtcpp2h
title: "P1: formats text/json/jsonl/yaml and the fdu.report/1 schema"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn1shjfrhncb9bhebyqx73
  - type: blocks
    target: is-01kzqn4rdq9vy4qvcve073rfhf
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:34:32.614Z
updated_at: 2026-08-11T05:36:21.430Z
---
CLI-feature format module over the structured Report. serde::Serialize derives enter the core crate (record in deny.toml). Renderers: text (files view prints one path per line and nothing else so it pipes into xargs; per-entry fields live in machine formats), json, jsonl, yaml. fdu.report/1 supersedes fdu.tree/2 - this spec explicitly authorizes the replacement the CLI UX plan forbade: schema, generator, root/root_raw, scan_started_at, generated_at, source, cache, complete, freshness, scope, selection, reports[]. YAML dependency: serde_yaml is unmaintained, so either a small first-party emitter over the already-structured Report or a vetted maintained crate through the 14-day cool-off; record the decision in deny.toml either way. Golden work per golden-testing-guidelines: fdu.report/1 fixture plus a schema-bump test that fails on unversioned change; tryscript goldens stay byte-stable (integers only, no floats in text) and normalize both timestamps through frontmatter patterns rather than eliding whole lines.
