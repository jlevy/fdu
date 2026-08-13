---
type: is
id: is-01kzwk20kyaxajq254tee8apts
title: "H59: Prototype a bounded-retention cache-off report path"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzw3te81j66eehy48rx2djv5
created_at: 2026-08-13T03:36:06.525Z
updated_at: 2026-08-13T03:48:38.150Z
---
Inspired by pdu 0.24.0: investigate whether a cache-off report can retain only the state required by the complete requested view set while producing byte-identical reports. This is design-gated because FDU's current views-read-an-index and one-scan-many-views principles forbid a hidden CLI-only shortcut. First deliver a semantic design or reject it; only then prototype in the library/Python surfaces with fail-closed fallback, exact provenance/errors, and a large RSS plus >=3% wall target.
