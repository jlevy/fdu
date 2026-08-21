---
type: is
id: is-01m0hg9fca6p7hs5xw2qp04315
title: Bump the content report schema for the analyzer-set field
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01m0hfrm2xbqzdx4avgegcvf0t
created_at: 2026-08-21T06:31:48.361Z
updated_at: 2026-08-21T07:15:54.510Z
closed_at: 2026-08-21T07:15:54.509Z
close_reason: Implemented on claude/fdu-content-axis; make check green (24 suites, 114 goldens).
---
The `analysis` block carries a lossy `"profile": "full"` label alongside the real
`analyzers` array. Under the set model the requested set is a list, so replace the label
with `"analyze": ["lines", "code", "words"]` -- what was asked for -- and keep
`analyzers` -- what ran, with versions.

This is a schema change to the content report only: `CONTENT_REPORT_SCHEMA` goes
`fdu.report/2` -> `fdu.report/3`. `REPORT_SCHEMA` (`fdu.report/1`, metadata-only) is
untouched, since the schema is already selected by whether analysis ran.

The schema-bump test must fail if the block changes without the version moving.
Mirror the same shape in the Python binding's report metadata.
