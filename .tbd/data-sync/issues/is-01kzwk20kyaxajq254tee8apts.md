---
type: is
id: is-01kzwk20kyaxajq254tee8apts
title: "H59: Prototype a bounded-retention cache-off report path"
kind: task
status: closed
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzw3te81j66eehy48rx2djv5
child_order_hints:
  - is-01kzx08n0ntz7ckxyv8q4msv23
  - is-01kzx08na3prrgaa9kw1dez8wv
  - is-01kzx0mvryf0a938vhwhy36cjv
  - is-01kzx0mvs4a2qgjh3vrc3rywmy
created_at: 2026-08-13T03:36:06.525Z
updated_at: 2026-08-13T09:49:41.884Z
closed_at: 2026-08-13T09:49:41.883Z
close_reason: Implemented, measured, documented, and fully validated as exp-040; H62-H65 track the remaining derived-scan layers.
---
Inspired by pdu 0.24.0: investigate whether a cache-off report can retain only the state required by the complete requested view set while producing byte-identical reports. This is design-gated because FDU's current views-read-an-index and one-scan-many-views principles forbid a hidden CLI-only shortcut. First deliver a semantic design or reject it; only then prototype in the library/Python surfaces with fail-closed fallback, exact provenance/errors, and a large RSS plus >=3% wall target.

## Notes

Accepted as exp-040. Internal fail-closed report planner selects one exact aggregate only for uncached unfiltered one-view summary; all other compositions retain the full index. Frozen heterogeneous 978,339-entry run: wall -14.56% [-18.55%, -9.04%], RSS -95.28%, identical stable report hash. Exact-final-binary mutation-free replications at 720,805 and 901,963 entries reproduced ~3x lower user CPU and 23-30x lower RSS but only 1.8-2.8% wall, so speed is topology-sensitive. No public Rust/Python API or CLI flag added. make check, 72 tryscript goldens, perf harness tests, schema, and flowmark --auto passed.
