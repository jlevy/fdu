---
type: is
id: is-01m0nzy0fdwze239vnncxkz1a4
title: "PR #42 review R18: the parity interpreter path is spelled literally in four places"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:07.597Z
updated_at: 2026-08-23T00:39:58.750Z
closed_at: 2026-08-23T00:39:58.750Z
close_reason: Fixed. PARITY_VENV_PYTHON and SMOKE_VENV_PYTHON in the Makefile; the CI job keeps its literal because it is a different file with no Makefile to read.
---
Makefile:182,187,190 and .github/workflows/ci.yml:213 each hard-code crates/fdu-py/.venv-*/bin/python. One Makefile variable would do.
