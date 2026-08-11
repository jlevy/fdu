---
type: is
id: is-01kzsa4tw6fqvemhmz3ckgcg86
title: "PR#6 C4: perf-test executes unpinned PyPI dependency outside lock and cool-off"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:38.469Z
updated_at: 2026-08-11T21:29:59.630Z
closed_at: 2026-08-11T21:29:59.630Z
close_reason: "Fixed and verified on PR #6; disposition posted to the PR"
---
Makefile:164-165. 'uv run --no-project --with pydantic' resolves at invocation time, not lockfile-frozen, not covered by exclude-newer. Violates SUPPLY-CHAIN-SECURITY.md. High.
