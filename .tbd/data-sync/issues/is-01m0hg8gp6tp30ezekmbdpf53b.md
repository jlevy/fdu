---
type: is
id: is-01m0hg8gp6tp30ezekmbdpf53b
title: Parse --analyze as a comma-delimited set with none/all totals
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01m0hfrm2xbqzdx4avgegcvf0t
created_at: 2026-08-21T06:31:16.933Z
updated_at: 2026-08-21T07:15:53.053Z
closed_at: 2026-08-21T07:15:53.052Z
close_reason: Implemented on claude/fdu-content-axis; make check green (24 suites, 114 goldens).
---
`--analyze` becomes a comma-delimited list parsed through the same `parse_list` helper
`--view` and `--kind` use: `none`, `lines`, `code`, `words`, `all`.

`none` and `all` are totals and must appear alone; combining either with another value is
a usage error naming the conflict. Duplicates are already rejected by `parse_list`.

Renames `basic` -> `lines` and `documents` -> `words` with no aliases retained, per the
precedent set by `--by-type` -> `--view types`. The interface is pre-release.

Mirror the same grammar in the Python binding's `parse_analysis_request`, so the two
surfaces accept exactly the same vocabulary (Principle 7).
