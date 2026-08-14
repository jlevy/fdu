---
type: is
id: is-01kzz2amst9cgp4sn6xtc0x0sz
title: Every experiment artifact fails its own schema because record.py writes an unquoted date
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:26.841Z
updated_at: 2026-08-14T03:00:06.486Z
closed_at: 2026-08-14T03:00:06.486Z
close_reason: "Added _is_iso_date to _needs_quoting so a written date survives as a string. Then requoted the committed artifacts: 51 date lines, plus bare 'off' argv items in exp-040 through exp-046 that an older emitter left unquoted and that read back as boolean False, misrepresenting the measured '--cache off'. All 51 artifacts now validate against Experiment.model_validate; before this none did. The softschema path had tolerated both, which is why the drift went unseen."
---
benchmarks/realtree/experiment.py declares date: str = Field(pattern=r'^\\d{4}-\\d{2}-\\d{2}$'), but record.py's _needs_quoting does not treat an ISO date as needing quotes, so the frontmatter carries a plain scalar.

Verified against docs/project/experiments/exp-050-decode-complete-utf-8-chunks-in-place.md on main: yaml.safe_load returns datetime.date(2026, 8, 13), and Experiment.model_validate rejects it with 'Input should be a valid string'. Every artifact in the ledger has the same defect.

It stays invisible because _validate prints 'skipped validation' and returns 0 whenever the softschema CLI is not on PATH, which is the normal case for record.

Fix the quoting so a written artifact round-trips through its own contract. Leave the skip-when-unavailable behaviour alone: main chose it deliberately and documents why.
