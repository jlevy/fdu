---
type: is
id: is-01kzwk20kyaxajq254tee8apts
title: "H59: Prototype a bounded-retention cache-off report path"
kind: task
status: in_progress
priority: 1
version: 7
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
updated_at: 2026-08-13T07:33:35.651Z
---
Inspired by pdu 0.24.0: investigate whether a cache-off report can retain only the state required by the complete requested view set while producing byte-identical reports. This is design-gated because FDU's current views-read-an-index and one-scan-many-views principles forbid a hidden CLI-only shortcut. First deliver a semantic design or reject it; only then prototype in the library/Python surfaces with fail-closed fallback, exact provenance/errors, and a large RSS plus >=3% wall target.
