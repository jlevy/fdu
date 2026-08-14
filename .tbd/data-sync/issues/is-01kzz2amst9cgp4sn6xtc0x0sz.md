---
type: is
id: is-01kzz2amst9cgp4sn6xtc0x0sz
title: Every experiment artifact fails its own schema because record.py writes an unquoted date
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:26.841Z
updated_at: 2026-08-14T02:41:26.841Z
---
benchmarks/realtree/experiment.py declares date: str = Field(pattern=r'^\\d{4}-\\d{2}-\\d{2}$'), but record.py's _needs_quoting does not treat an ISO date as needing quotes, so the frontmatter carries a plain scalar.

Verified against docs/project/experiments/exp-050-decode-complete-utf-8-chunks-in-place.md on main: yaml.safe_load returns datetime.date(2026, 8, 13), and Experiment.model_validate rejects it with 'Input should be a valid string'. Every artifact in the ledger has the same defect.

It stays invisible because _validate prints 'skipped validation' and returns 0 whenever the softschema CLI is not on PATH, which is the normal case for record.

Fix the quoting so a written artifact round-trips through its own contract. Leave the skip-when-unavailable behaviour alone: main chose it deliberately and documents why.
