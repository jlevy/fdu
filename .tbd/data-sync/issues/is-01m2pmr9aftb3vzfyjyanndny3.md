---
type: is
id: is-01m2pmr9aftb3vzfyjyanndny3
title: "Explicit core models: request, execution plan, stored state, provenance, answer"
kind: epic
status: open
priority: 0
version: 11
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - release
  - design
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
child_order_hints:
  - is-01m2pmr9n3mq4nb2r1pc328qpz
  - is-01m2pmr9ytx0ye8d701mr5vp9s
  - is-01m2pmra8yqrcxg27kc6ezg9vd
  - is-01m2pmram44dgp78vm6xq4w7k7
  - is-01m2pmrbxvnerxyhnjhrswy1ye
  - is-01m2pmrcb8he4a8a54zt957vcs
  - is-01m2pmrcrmjrm62bm1x3mxwgvm
  - is-01m2phzn814exmf4ty5vw6zha0
  - is-01m2pj0f459s8ad1efzyn2qmbq
  - is-01m2pmrn6ka9kt7f4kcjcmvn8c
created_at: 2026-09-17T02:57:23.790Z
updated_at: 2026-09-17T02:57:35.954Z
---
Ships in 0.1.0 (maintainer decision 2026-09-17). Every key concept gets one explicit, typed model in
fdu-core, consumed by every route and surface, so caching improves performance and never changes
semantics. Invariant: for every request and history, the answer equals the cold answer apart from
provenance, or the run fails with a named reason, or cache-only labels itself stale. Scope may shrink
only by the plan's listed deferrals. Evidence and design: the linked plan; the principles "Model Every
Key Concept Explicitly, in One Place" and "Caching Improves Performance, Never Semantics".
