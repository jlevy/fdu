---
type: is
id: is-01kzs52fyqhmvw2dz2jkm4hqd4
title: Surface provenance in Report, formats, and Python
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzs52gqgbksbwf8wme30ymrp
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:33:58.867Z
updated_at: 2026-08-11T19:33:59.661Z
---
Provenance is a library property, so every consumer reads it from the same place and the CLI displays it rather than inventing it (composable-CLI principle 7). Report rows carry provenance; text, json, jsonl and yaml serialise it with status as a STRING so future Status values are additive under the schema-versioning rules; Python exposes the same struct. Golden/schema tests for each format.
