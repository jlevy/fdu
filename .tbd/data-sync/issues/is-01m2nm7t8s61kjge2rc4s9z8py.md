---
type: is
id: is-01m2nm7t8s61kjge2rc4s9z8py
title: "PR #71 review 71-1: opened-root plan names one P0 child of fdu-2lkf where there are three"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2nm7m7kc7hna3r0cr3hyxf3
created_at: 2026-09-16T17:29:09.656Z
updated_at: 2026-09-16T17:37:32.355Z
closed_at: 2026-09-16T17:37:32.355Z
close_reason: "f84e320 (PR #71): the opened-root plan names all three P0 children of fdu-2lkf at both sites: etfj closed at PR #48's merge, 1onj in PR #63, pro1 in progress."
resolution: null
duplicate_of: null
---
docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md:1338-1340 and :2944 @4c70e10 (PR #71). "The P0 child of fdu-2lkf ... fdu-1onj closed" is singular, but fdu-2lkf has three P0 children: fdu-etfj (closed at PR #48's merge), fdu-1onj (closed, PR #63) and fdu-pro1 (in progress). State all three.
