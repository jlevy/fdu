---
type: is
id: is-01m2vn6dv3j0g953gyfyrx1vzv
title: Snake-case installed-CLI QA harness and apply Python guidelines
kind: bug
status: closed
priority: 1
version: 3
spec_path: tests/qa/cli-installed-e2e.qa.md
labels: []
dependencies: []
parent_id: is-01m2v3j2nf6djtd4bcbj6mrevh
created_at: 2026-09-19T01:41:19.330Z
updated_at: 2026-09-19T01:44:03.796Z
closed_at: 2026-09-19T01:44:03.794Z
close_reason: "Renamed scripts/run-installed-cli-qa.py to scripts/run_installed_cli_qa.py and applied python-rules / python-modern-guidelines on PR #90 (2f955a58)."
resolution: null
duplicate_of: null
---
scripts/run-installed-cli-qa.py is kebab-case (forbidden). Rename to snake_case and apply python-rules / python-modern-guidelines to the PR #90 harness: imports, pathlib, named constants, atomic result writes, no Optional, stdlib-only sequential script (not make check). Update playbook, dated report, and PR body references.
