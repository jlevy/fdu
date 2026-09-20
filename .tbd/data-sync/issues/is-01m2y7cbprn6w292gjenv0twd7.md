---
type: is
id: is-01m2y7cbprn6w292gjenv0twd7
title: Enhance help examples with README workflows and age/size directory searches
kind: task
status: open
priority: 2
version: 2
labels: []
dependencies:
  - type: blocks
    target: is-01m2y7cf9fsawdtq8p5grnr3nq
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
created_at: 2026-09-20T01:37:36.982Z
updated_at: 2026-09-20T01:38:18.020Z
---
Add discoverable --help examples consistent with README: .venv or venv older than 7d, node_modules and Cargo target older than 30d, union include patterns, size and actual age in text, JSON output and oldest-first sorting. Teach directories vocabulary and root/aggregate filter semantics without changing existing files output. Review help goldens.
