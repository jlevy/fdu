---
type: is
id: is-01kzyv83g6px1zbkrp5cj3bck2
title: Make language summaries metadata-only by default
kind: bug
status: closed
priority: 1
version: 5
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-14T00:37:43.551Z
updated_at: 2026-08-14T01:00:23.063Z
closed_at: 2026-08-14T01:00:23.057Z
close_reason: Implemented metadata-only language summaries with additive LOC, updated docs and Rust/Python/golden coverage, and made the human golden filesystem-neutral after Windows CI exposed allocated-size differences; full make check passes.
---
Allow fdu --view languages PATH without --analyze. Use path-only exact-name/extension classification and byte shares when code analysis is absent; preserve LOC/code-line shares when --analyze code or full is present. Update help, README, portable skill, query validation, goldens, global installation, and CI.
