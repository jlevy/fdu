---
type: is
id: is-01m4ws7r70qggtac846q9ye37q
title: Measure and simplify Python page and change-batch conversion
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-10-14T08:42:35.092Z
updated_at: 2026-10-14T08:42:35.092Z
---
Future improvement from PR205 copy audit: current Tree/Flat/Lookup/change results are owned native values converted to temporary PyDict/PyList graphs and then typed Python models. Measure transient peak memory, conversion, scalar reuse and teardown; compare direct model construction or an immutable native batch handle preserving path encoding, ordering, version/coverage/refusal semantics and ordinary ergonomics. Preserve coarse batch operations and avoid per-field call overhead. Complement fdu-5ve4 report/lazy handles and fdu-g20s optional buffers; no zero-copy claim for current batches.
