---
type: is
id: is-01kzs52gqgbksbwf8wme30ymrp
title: "CLI display of provenance: quiet when fresh, explicit when not"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:33:59.661Z
updated_at: 2026-08-11T19:33:59.661Z
---
An 'as of' header and per-row markers. The common case stays silent: a one-shot command verifies before printing, so every row is Scanned or Revalidated and there is nothing to annotate. Provenance becomes visible exactly where it should - under --cache only (every row Cached, header says as of when), under --allow-partial (incomplete subtrees marked), and in any future progress mode. Tryscript goldens for each.
