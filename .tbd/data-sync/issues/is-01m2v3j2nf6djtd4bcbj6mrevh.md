---
type: is
id: is-01m2v3j2nf6djtd4bcbj6mrevh
title: Installed CLI end-to-end QA playbook and sequential harness
kind: task
status: open
priority: 2
version: 2
spec_path: tests/qa/cli-installed-e2e.qa.md
labels: []
dependencies: []
created_at: 2026-09-18T20:33:06.734Z
updated_at: 2026-09-18T20:38:33.997Z
---
Add tests/qa/cli-installed-e2e.qa.md plus scripts/run-installed-cli-qa.py. Sequential PATH-binary QA: view matrix, cache off vs on for --analyze=code and --analyze=lines, optional medium tree, bounded Library. Dated numbers in docs/project/reports/report-2026-09-18-cli-installed-qa.md. Not part of make check.

## Notes

PR https://github.com/jlevy/fdu/pull/90 on docs/cli-installed-qa-playbook (50d768e7).
First run (fdu 0.1.0-dev+gcb9666a2a): cache-on --analyze=code 3.12s → 0.88s (34,145 cached); --cache=off stayed 0 cached. Library bounded escalate: no SIGKILL.
