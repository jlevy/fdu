---
type: is
id: is-01m2k28bn0m1f1em1zb5a5sb4j
title: "PR #61 review PR61-PY-2: the free-threaded guard accepts uv's full request form cpython-3.14t-macos-aarch64-none"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m2k27z25tt9ygs4c1nchhzez
created_at: 2026-09-15T17:36:24.223Z
updated_at: 2026-09-15T17:43:33.237Z
closed_at: 2026-09-15T17:43:33.236Z
close_reason: "07efcd2: the guard splits WHEEL_PYTHON on - and matches a version ending in t or td, plus freethreaded; Node test adds cpython-3.14t-macos-aarch64-none and 3.14td (both fail on the old guard) and a GIL-enabled full form."
resolution: null
duplicate_of: null
---
PR #61, delta review 5213560245. Makefile:216 @ 4db083b. The filter matches a word ending in digit+t, so the t followed by - in the full request form passes, and uv resolves it to a free-threaded 3.14. Optional; do it.
