---
type: is
id: is-01m2e9nqdh5kzg3hg5jbbrpc26
title: Document durable disk-usage checkpoints and daily delta workflow
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
created_at: 2026-09-13T21:09:50.127Z
updated_at: 2026-09-13T21:22:30.291Z
---

## Notes

Precommit review: docs-only, five files on isolated branch codex/disk-usage-checkpoints from main b75bf85a33ed. Corrected current auto/read-only snapshot bypass and transient summary persistence, v3 cursor collision with open stack, FullHistory overlap and unsupported completeness inference, the next-day age-gate conflict, O(N) persistence costs, engine ownership, and fixed-baseline accounting. Added fdu-uwhl and fdu-8ybz; preserved and annotated historical replay issues. Design assessment: existing engine facts/reducers plus immutable checkpoints and native journal scope discovery fit the surface contract; Spotlight remains an optional hint. No remaining actionable prose/design review findings. Markdown, footer, whitespace, and 60 local-link checks pass. Full make check is running with artifacts on external storage; no engine behavior changes or benchmark claim.
