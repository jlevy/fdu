---
type: is
id: is-01m4wryk6q6f2dw2x9g4zn5gr0
title: Evaluate optional bulk buffer or columnar Python export
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-10-14T08:37:35.051Z
updated_at: 2026-10-14T08:37:35.051Z
---
Future improvement from PR205 boundary review: measure whether full-inventory consumers benefit from owned immutable numeric/path buffers, memoryview/NumPy or Arrow record batches. Distinguish native packing copies from zero-copy sharing into Python; no current buffer export is claimed. Define lossless path encoding, nullability/coverage/version/order schema, lifetime across refresh/close, and release ownership. Avoid pinning mutable index locks or unbounded old snapshots. Benchmark against selected/paged current API before adding dependencies; preserve convenient models and explicit complete-result use.
