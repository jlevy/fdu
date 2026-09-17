---
type: is
id: is-01m2pmrcrmjrm62bm1x3mxwgvm
title: "Session rules: watch keeps or refuses tiers; opened and Python Index validate requests"
kind: task
status: open
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - design
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01kzypf1yd2v4g8q8tk2v1xmxs
created_at: 2026-09-17T02:57:27.315Z
updated_at: 2026-09-17T02:57:35.954Z
---
Watch sessions serve only tiers they keep current: refuse content analysis at session creation (engine, not CLI)
and refuse --watch --cache only; repaint provenance from the provenance model. Opened reads validate through the
request model (documents without analysis is refused). Python Index carries its request; a report with a
different content axis is projected or refused. Relative time windows in sessions get one explicit definition.
