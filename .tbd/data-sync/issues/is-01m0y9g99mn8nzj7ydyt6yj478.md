---
type: is
id: is-01m0y9g99mn8nzj7ydyt6yj478
title: Clarify OpenedIndex as the direct public API, not a facade layer
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T05:43:19.076Z
updated_at: 2026-08-26T06:43:18.210Z
closed_at: 2026-08-26T06:43:18.192Z
close_reason: Updated the durable engine architecture and active file/function plan so OpenedIndex is the direct live-root API, OpenedState is private data and synchronization only, and no facade, parallel Owner service, mirror method surface, or separate session object is permitted; committed as 1927489 with all 19 CI checks green.
resolution: null
duplicate_of: null
---
Revise the durable engine architecture and active implementation plan so OpenedIndex is the direct live-root API backed by private shared state, with no parallel Owner service, mirror method surface, or forwarding facade. Preserve the single shared lifetime and clone-wide shutdown invariants.
