---
type: is
id: is-01kzqn66p0pmck4yg6pexhww2z
title: "P4: distill design principles into a durable guide"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:37:08.799Z
updated_at: 2026-08-11T05:37:08.799Z
---
Phase 4 is small but not optional: the principles must outlive the spec. Distill the Goals and Design Principles as actually implemented, including any amendments iteration forced, into docs/project/guides/fdu-design-principles.md following common-doc-guidelines with the standard footer: the five axes and what belongs to each, the delta contract (producers observe, index consumes, views read), cache honesty (source/freshness/complete labeling; never silently stale), the CLI-invents-nothing parity rule, the subsumption checklist against du/dust/dut/diskus/fd/find, and the watch efficiency contract (event-driven, interval is render-only, polling only as the NFS/FUSE fallback). Run the end-of-plan parity review - what, if anything, lives only in cli.rs - and record its outcome in the guide. Then point AGENTS.md, README, and the architecture references at it, move the spec to done, and reconcile the subsumed beads per the spec's open question 3 (fdu-oqoy adaptive width and gitignore display, fdu-jej9 JSONL and schema docs) with maintainer sign-off before closing or re-parenting either.
