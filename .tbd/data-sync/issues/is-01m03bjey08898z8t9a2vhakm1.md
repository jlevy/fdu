---
type: is
id: is-01m03bjey08898z8t9a2vhakm1
title: "Cache layers and defaults: make the snapshot participate only when it pays"
kind: epic
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-15-fdu-cache-layers-and-defaults.md
labels:
  - performance
  - cli
  - cache
dependencies: []
child_order_hints:
  - is-01m03bjv6fd4964hf1201aer1c
  - is-01m03b8f0qwm5yp2kv0cv0t0nn
  - is-01m03wk6dz5xq12ywyf4227n2n
  - is-01m03y2wrr57zxh6g5n7zsj8n3
created_at: 2026-08-15T18:39:57.888Z
updated_at: 2026-08-16T00:03:30.711Z
---
Umbrella for the cost model under the three product layers (parity / cached / progressive). Phase 1 (plan selection + status parity + docs) is implemented; Phase 2 (persistence gate) needs macOS measurement and must land before Phase 1 ships as a default, because a summary run no longer populates a snapshot for a later --cache only read. See the spec.
