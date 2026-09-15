---
type: is
id: is-01m2k28c3mgcyj655tr20j2tpm
title: "PR #61 review PR61-PY-3: in make check the free-threaded refusal comes after the whole Rust gate"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m2k27z25tt9ygs4c1nchhzez
created_at: 2026-09-15T17:36:24.690Z
updated_at: 2026-09-15T17:44:14.336Z
closed_at: 2026-09-15T17:44:14.336Z
close_reason: "0f6a6d4: wheel-python is check's second prerequisite; make -n check UV_PYTHON=3.14t refuses on line 27 (was 92) after only the uv-version recipe; Node test compares the output with make -n uv-version and fails on the old order."
resolution: null
duplicate_of: null
---
PR #61, delta review 5213560245. Makefile:114 @ 4db083b. make -n check UV_PYTHON=3.14t prints 92 lines and refuses on the last, after cargo fmt, clippy, and test. Run wheel-python among check's first prerequisites. Optional; do it.
