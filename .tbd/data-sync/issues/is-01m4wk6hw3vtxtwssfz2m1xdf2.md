---
type: is
id: is-01m4wk6hw3vtxtwssfz2m1xdf2
title: Unify deep report decoding across Python lifecycles
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-10-14T06:57:04.386Z
updated_at: 2026-10-14T06:57:04.386Z
---
Astra review A3: opened_binding.rs ProjectionResult::Report directly calls json.loads, bypassing _api._loads_object iterative RecursionError fallback. Source-based expected failure on deep opened trees; not runtime-reproduced. Use common decoder or remove native JSON parsing; avoid recursive Report clone by moving owned result. Acceptance: equivalent deep detached/opened reports decode with identical shape; isolated recursion-limit regression or native report seam; make check.
