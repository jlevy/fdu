---
type: is
id: is-01m0pg6ven5sm1hhz3rpbyf4qb
title: Record tree provenance for the 64 artifacts that predate the rule
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-experiment-evidence-scope.md
labels: []
dependencies: []
created_at: 2026-08-23T05:06:34.581Z
updated_at: 2026-08-23T05:36:11.394Z
---
Subject gained tree_provenance and tree_reconstructible in exp-065's change, and make perf-record now takes the flags. Of the artifacts recorded before that, only exp-064 named how its subject was built (and omitted the entry-count argument that determined it). The ledger now prints 'Provenance unrecorded' for the rest, which is accurate but leaves 20 subjects nobody else can obtain. Backfill what is knowable -- the generated corpora especially, where the recipe is a command -- and state plainly where no recipe exists (live-workspace-*, the metabrowser clones). Do not invent a recipe that was not used.
