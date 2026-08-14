---
type: is
id: is-01kzzbbjxb78m4rde2gb10kmjk
title: Free producer allocations in the producing thread, not the consumer
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T05:19:14.858Z
updated_at: 2026-08-14T05:19:14.858Z
---
H51 (move paths instead of cloning) and H62 (worker-local reduction) both reduced the NUMBER of allocations on the aggregate tier and both were refuted on wall time. A mimalloc global allocator changes none of the counts and measures -23.0 percent [-28.4, -16.7] on the same tier. Those two facts together say the cost is not allocation volume but glibc malloc's cross-thread free path specifically: scan workers allocate paths and observation batches, the single consumer frees them, and that is the pattern glibc handles worst. A structural fix would change WHO frees rather than how much is allocated - returning drained batch buffers to their producing worker for reuse, so each arena is allocated and freed on one thread. That would capture the same win without a dependency, and unlike mimalloc it would not cost +139 percent peak RSS on the tier whose selling point is low memory. Screen against the mimalloc number: anything below about 20 percent is not capturing the same cost.
