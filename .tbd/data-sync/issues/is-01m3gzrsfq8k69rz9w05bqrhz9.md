---
type: is
id: is-01m3gzrsfq8k69rz9w05bqrhz9
title: Reconcile tbd managed-skill drift checks with required Markdown formatting
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-09-27-cli-and-skill-followups.md
labels: []
dependencies: []
created_at: 2026-09-27T08:30:09.910Z
updated_at: 2026-09-27T08:30:09.910Z
---
tbd 0.9.0 setup produces current portable/Claude skills, then required make docs-format changes YAML presentation and prose wrapping. tbd doctor reports both as stale although parsed frontmatter and whitespace-normalized body match tbd skill. Decide formatter ownership or upstream semantic drift normalization; keep actual content drift detectable. Reproduction and normalized comparison recorded in the 2026-09-27 tracking review. Do not disable the repository documentation check.
