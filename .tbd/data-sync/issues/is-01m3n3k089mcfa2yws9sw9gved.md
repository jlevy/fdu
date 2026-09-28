---
type: is
id: is-01m3n3k089mcfa2yws9sw9gved
title: Build the deep-render fixture in one batch (about 12 s off every make test)
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T22:53:52.264Z
updated_at: 2026-09-28T22:53:52.264Z
---
report_format::tests::deep_rendering_is_stack_safe builds its 1,024-level fixture one directory per batch, and each batch re-checks every ancestor, so construction is cubic in depth: 17.8 s of the child's ~21 s. One batch measured 5.4 s. Found while fixing fdu-xsg1 (04d640e3).
