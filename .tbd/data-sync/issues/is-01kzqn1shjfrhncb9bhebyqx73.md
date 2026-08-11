---
type: is
id: is-01kzqn1shjfrhncb9bhebyqx73
title: "P1: CLI rework to five axes with docs and benchmark manifests"
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn2fn6p7qcmp31j87qesak
  - type: blocks
    target: is-01kzqn2s3rwkxhb8ag9v4e6t24
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:34:44.273Z
updated_at: 2026-08-11T16:01:40.824Z
---
cli.rs shrinks to parsing flags into (ScanConfig, CachePolicy, Query, Format) plus stream routing; the current private rendering methods on Cli move behind query/format types with their own unit tests. Replaced flags, no aliases (pre-release): --by-type -> --view types, --json -> --format json, --apparent-size -> --size apparent, --number -> -n/--limit, --max-depth -> --scan-depth, --no-cache -> --cache off. Grammar conventions applied uniformly: closed vocabularies (--view, --kind) are comma lists that split, trim, reject empty tokens, name the valid values on an unknown token, error on duplicates, and render views in the given order; open pattern values (--include/--exclude) are repeatable flags. Bounded values accept 'all'; --depth 0 keeps du's root-totals meaning. The same change updates AFTER_HELP, SKILL.md, README, all four tests/golden/*.tryscript.md files, and every benchmark manifest carrying the old argv (benchmarks/scenarios.json cli-human and cli-json jobs, benchmarks/realtree/measure.py job argv) - flags are benchmark identity per Principle 11. Exit contract unchanged. Close the phase with the parity review: record what, if anything, lives only in cli.rs.
