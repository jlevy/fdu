---
type: is
id: is-01m03bjey08898z8t9a2vhakm1
title: "Cache layers and defaults: make the snapshot participate only when it pays"
kind: epic
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/done/plan-2026-08-15-fdu-cache-layers-and-defaults.md
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
updated_at: 2026-08-23T02:14:25.975Z
closed_at: 2026-08-23T02:12:22.254Z
close_reason: |
  All three phases resolved; the epic has no remaining children.

  Phase 1 (plan selection) and Phase 3 (stop reading where the snapshot cannot pay) landed.
  Phase 2 was closed by measurement rather than implemented: the APFS trial argued against a
  size threshold rather than supplying one, so SNAPSHOT_MIN_ENTRIES stays None as a measured
  decision (fdu-hvs5, closed with the evidence).

  The one bead still citing this plan, fdu-wu6w (prefer-cache tier for progressive UIs),
  belongs to fdu-wpa0 and is specified by the progressive-results plan; its spec pointer has
  been corrected. The cost model this plan established -- a snapshot earns its keep when it
  avoids expensive work, not when it mirrors a walk that still has to happen -- is what the
  campaign-2 plan's warm posture rests on, and the floor measurement reconfirmed it
  independently.
---
Umbrella for the cost model under the three product layers (parity / cached / progressive). Phase 1 (plan selection + status parity + docs) is implemented; Phase 2 (persistence gate) needs macOS measurement and must land before Phase 1 ships as a default, because a summary run no longer populates a snapshot for a later --cache only read. See the spec.
