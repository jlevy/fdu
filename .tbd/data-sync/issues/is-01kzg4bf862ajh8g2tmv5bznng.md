---
type: is
id: is-01kzg4bf862ajh8g2tmv5bznng
title: "CLI agent surface: stable JSON schema, exit codes, help completeness"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:28:14.597Z
updated_at: 2026-08-08T07:28:14.597Z
---
Agents get --help as the complete source of truth, stable machine-readable output whose schema is versioned with the tool, meaningful exit codes, and no interactive surprises (no pager, no prompts).

Present: fdu.tree/1 schema with generator/root/source/complete/errors/by_extension/tree, hand-rolled and valid.
Still needed: JSONL streaming mode for large trees (one object per entry, so a consumer does not buffer the whole tree), documented exit codes distinguishing 'partial results due to unreadable paths' from 'failed', a schema doc, and a golden test that fails when the schema changes without a version bump.

Because warm queries are milliseconds, agents can call this freely — tally a tree by type, top 20 largest, what changed in the last hour. A small agent skill documenting usage would give the CLI a second consumer that keeps it honest.
