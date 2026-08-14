---
type: is
id: is-01kzyr76mpg2emh1sf1zktb7y4
title: Make common FDU cost-layer recipes obvious in help and README
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-13T23:44:48.274Z
updated_at: 2026-08-14T00:03:56.046Z
closed_at: 2026-08-14T00:03:56.046Z
close_reason: Implemented in c2b646c; full make check and all 16 required PR checks pass.
---
Organize four prominent recipes with exact commands and costs: language/SLOC analysis, metadata-only file types, folder-size tree, and exact totals-only summary. Keep --help and the top-level README consistent and cover the help text with golden tests.

## Notes

Implemented a four-command common-report map in CLI help, the top-level README, and the portable skill. Verified all four commands on one fixture; the existing summary text contract includes directory count in addition to bytes and files. Covered by the 97-scenario CLI golden suite and full make check.
