---
type: is
id: is-01m4wk6k23r8bcbc5tc431hfsj
title: Benchmark installed Python boundary and agent-loop workloads
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-10-14T06:57:05.602Z
updated_at: 2026-10-14T06:57:05.602Z
---
Astra review A4: python-open-query is only schema vocabulary; native query/render probes and wheel CLI startup bypass Python report conversion. Add installed-wheel public API benchmarks varying inventory and selected result cardinalities independently. Separate Rust work, rendering, decode/models, as_dict, retention/teardown; record RSS plus Python allocations, GC callback duration, GIL/event-loop responsiveness, rows/bytes, cold vs retained reads, consume/drop vs retain pages, deep/wide/raw-path cases and churn. Validate answers and pin artifact identity. Use external task scratch and performance-loop evidence protocol.
